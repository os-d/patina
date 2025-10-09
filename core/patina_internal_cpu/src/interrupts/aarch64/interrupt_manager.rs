//! AARCH64 Interrupt manager
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use core::arch::{asm, global_asm};
use patina::{
    base::{UEFI_PAGE_MASK, UEFI_PAGE_SIZE},
    bit,
    component::service::IntoService,
    error::EfiError,
};
use patina_paging::{PageTable, PagingType};
use patina_stacktrace::StackTrace;

#[cfg(all(not(test), target_arch = "aarch64"))]
use crate::interrupts::aarch64::sysreg::{read_sysreg, write_sysreg};
use crate::interrupts::{EfiSystemContext, HandlerType, InterruptManager, exception_handling::FaultAllocator};
use crate::interrupts::{disable_interrupts, enable_interrupts};

#[cfg(all(not(test), target_arch = "aarch64"))]
use crate::interrupts::aarch64::gic_manager::get_current_el;

global_asm!(include_str!("exception_handler.asm"));

#[cfg(all(not(test), target_arch = "aarch64"))]
// extern "efiapi" fn AsmGetVectorAddress(index: u64);
unsafe extern "C" {
    static exception_handlers_start: u64;
    static sp_el0_end: u64;
}

/// AARCH64 Implementation of the InterruptManager.
#[derive(Default, Copy, Clone, IntoService)]
#[service(dyn InterruptManager)]
pub struct InterruptsAarch64 {}

impl InterruptsAarch64 {
    /// Creates a new instance of the AARCH64 implementation of the InterruptManager.
    pub const fn new() -> Self {
        Self {}
    }

    /// Initializes the hardware and software structures for interrupts and exceptions.
    ///
    /// This routine will initialize the architecture and platforms specific mechanisms
    /// for interrupts and exceptions to be taken. This routine may install some
    /// architecture specific default handlers for exceptions.
    ///
    pub fn initialize(&mut self) -> Result<(), EfiError> {
        // Initialize exception entrypoint
        initialize_exception()?;

        self.register_exception_handler(0, HandlerType::UefiRoutine(synchronous_exception_handler))
            .expect("Failed to install default exception handler!");

        Ok(())
    }
}

impl InterruptManager for InterruptsAarch64 {}

fn enable_fiq() {
    unsafe {
        asm!("msr   daifclr, 0x01", "isb sy", options(nostack));
    }
}

fn disable_fiq() {
    unsafe {
        asm!("msr   daifset, 0x01", "isb sy", options(nostack));
    }
}

fn get_fiq_state() -> Result<bool, EfiError> {
    #[cfg(all(not(test), target_arch = "aarch64"))]
    {
        let daif = unsafe { read_sysreg!(daif) };
        Ok(daif & 0x40 == 0)
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        Err(EfiError::Unsupported)
    }
}

fn enable_async_abort() {
    #[cfg(all(not(test), target_arch = "aarch64"))]
    {
        unsafe {
            asm!("msr   daifclr, 0x04", "isb sy", options(nostack));
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        unimplemented!()
    }
}

fn initialize_exception() -> Result<(), EfiError> {
    // Set the stack pointer for EL0 to be used for synchronous exceptions
    #[cfg(all(not(test), target_arch = "aarch64"))]
    unsafe {
        let mut sp_el0_reg = &sp_el0_end as *const _ as u64;
        sp_el0_reg &= !0x0F;
        asm!("msr sp_el0, {}", in(reg) sp_el0_reg, options(nostack));

        let mut hcr = read_sysreg!(hcr_el2) as u64;
        hcr = hcr as u64 | 1 << 27; // Enable TGE
        write_sysreg!(hcr_el2, hcr);
    }

    // Program VBar
    #[cfg(all(not(test), target_arch = "aarch64"))]
    {
        let vec_base = unsafe { &exception_handlers_start as *const _ as u64 };
        let current_el = get_current_el();
        match current_el {
            0xC => unsafe { write_sysreg!(vbar_el1, vec_base, "isb sy") },
            0x08 => unsafe { write_sysreg!(vbar_el2, vec_base, "isb sy") },
            0x04 => unsafe { write_sysreg!(vbar_el3, vec_base, "isb sy") },
            _ => panic!("Invalid current EL {}", current_el),
        };
    }

    let fiq = get_fiq_state();

    disable_interrupts();
    disable_fiq();

    if fiq.is_ok_and(|fiq_b| fiq_b) {
        enable_fiq();
    }

    // We will always enable interrupt when initializing the exception manager.
    enable_interrupts();
    enable_async_abort();

    Ok(())
}

/// Default handler for synchronous exceptions.
extern "efiapi" fn synchronous_exception_handler(_exception_type: isize, context: EfiSystemContext) {
    let aarch64_context = unsafe { context.system_context_aarch64.as_ref().unwrap() };

    log::error!("");
    log::error!("EXCEPTION: Synchronous Exception");
    log::error!("");
    log::error!(
        "ESR: {:#X?}, ELR: {:#X?}, SPSR: {:#X?}, FAR: {:#X?}",
        aarch64_context.esr,
        aarch64_context.elr,
        aarch64_context.spsr,
        aarch64_context.far
    );

    log::error!("");

    // determine if this was a page fault
    let ec = (aarch64_context.esr >> 26) & 0x3F;
    let iss = aarch64_context.esr & 0xFFFFFF;
    let page_fault = ec == 0x20 || ec == 0x21 || ec == 0x24 || ec == 0x25;
    if ec == 0x20 || ec == 0x21 {
        // Instruction Abort from a lower EL or same EL
        log::error!("Page Fault (Instruction Abort)");
    } else if ec == 0x24 || ec == 0x25 {
        // Data Abort from a lower EL or same EL
        log::error!("Page Fault (Data Abort)");
    }

    log::error!("");

    log::error!("General-Purpose Registers");
    log::error!(
        "{:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}",
        "x0",
        aarch64_context.x0,
        "x1",
        aarch64_context.x1,
        "x2",
        aarch64_context.x2,
        "x3",
        aarch64_context.x3
    );
    log::error!(
        "{:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}",
        "x4",
        aarch64_context.x4,
        "x5",
        aarch64_context.x5,
        "x6",
        aarch64_context.x6,
        "x7",
        aarch64_context.x7
    );
    log::error!(
        "{:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}",
        "x8",
        aarch64_context.x8,
        "x9",
        aarch64_context.x9,
        "x10",
        aarch64_context.x10,
        "x11",
        aarch64_context.x11
    );
    log::error!(
        "{:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}",
        "x12",
        aarch64_context.x12,
        "x13",
        aarch64_context.x13,
        "x14",
        aarch64_context.x14,
        "x15",
        aarch64_context.x15
    );
    log::error!(
        "{:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}",
        "x16",
        aarch64_context.x16,
        "x17",
        aarch64_context.x17,
        "x18",
        aarch64_context.x18,
        "x19",
        aarch64_context.x19
    );
    log::error!(
        "{:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}",
        "x20",
        aarch64_context.x20,
        "x21",
        aarch64_context.x21,
        "x22",
        aarch64_context.x22,
        "x23",
        aarch64_context.x23
    );
    log::error!(
        "{:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}",
        "x24",
        aarch64_context.x24,
        "x25",
        aarch64_context.x25,
        "x26",
        aarch64_context.x26,
        "x27",
        aarch64_context.x27
    );
    log::error!(
        "{:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}  {:>4}: {:#018X}",
        "x28",
        aarch64_context.x28,
        "fp",
        aarch64_context.fp,
        "lr",
        aarch64_context.lr,
        "sp",
        aarch64_context.sp
    );

    log::error!("");

    if page_fault {
        // make sure the FAR is valid before we dump the page table
        if iss & bit!(10) == 0 {
            dump_pte(aarch64_context.far);
        } else {
            log::error!("FAR not valid, not dumping PTE");
        }
    }

    log::debug!("Full Context: {aarch64_context:#X?}");

    log::error!("Dumping Exception Stack Trace:");
    if let Err(err) = unsafe { StackTrace::dump_with(aarch64_context.elr, aarch64_context.sp) } {
        log::error!("StackTrace: {err}");
    }

    // no need to print anything here, we already did
    panic!("");
}

fn dump_pte(far: u64) {
    let ttbr0_el2 = unsafe { read_sysreg!(ttbr0_el2) };

    if let Ok(pt) = unsafe {
        patina_paging::aarch64::AArch64PageTable::from_existing(ttbr0_el2, FaultAllocator {}, PagingType::Paging4Level)
    } {
        let _ = pt.dump_page_tables(far & !(UEFI_PAGE_MASK as u64), UEFI_PAGE_SIZE as u64);
        log::error!("");
    }
}
