//! X64 Interrupt module
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use core::arch::asm;
use patina::error::EfiError;
use patina_pi::protocols::cpu_arch::EfiSystemContext;
use patina_stacktrace::StackTrace;

cfg_if::cfg_if! {
    if #[cfg(all(target_os = "uefi", target_arch = "x86_64"))] {
        mod interrupt_manager;
        pub use interrupt_manager::InterruptsX64;
    } else if #[cfg(feature = "doc")] {
        pub use interrupt_manager::InterruptsX64;
        mod interrupt_manager;
    }
}

pub type ExceptionContextX64 = r_efi::protocols::debug_support::SystemContextX64;

impl super::EfiSystemContextFactory for ExceptionContextX64 {
    fn create_efi_system_context(&mut self) -> EfiSystemContext {
        EfiSystemContext { system_context_x64: self as *mut _ }
    }
}

impl super::EfiExceptionStackTrace for ExceptionContextX64 {
    fn dump_stack_trace(&self) {
        if let Err(err) = unsafe { StackTrace::dump_with(self.rip, self.rsp) } {
            log::error!("StackTrace: {err}");
        }
    }

    fn dump_system_context_registers(&self) {
        log::error!(
            "Control Registers:\n \
            CR0: {:#X?} CR2: {:#X?} CR3: {:#X?} CR4: {:#X?}\n \
            RIP: {:#X?}  CS: {:#X?}  SS: {:#X?}  DS: {:#X?}\n \
            RSP: {:#X?} RFLAGS: {:#X?}\n",
            self.cr0,
            self.cr2,
            self.cr3,
            self.cr4,
            self.rip,
            self.cs,
            self.ss,
            self.ds,
            self.rsp,
            self.rflags
        );

        log::error!("");

        log::error!("General-Purpose Registers");
        log::error!(
            "{:>4}  {:#018X}   {:>4}  {:#018X}   {:>4}  {:#018X}  {:>4}  {:#018X}",
            "RAX",
            self.rax,
            "RBX",
            self.rbx,
            "RCX",
            self.rcx,
            "RDX",
            self.rdx
        );

        log::error!(
            "{:>4}  {:#018X}   {:>4}  {:#018X}   {:>4}  {:#018X}  {:>4}  {:#018X}",
            "RSI",
            self.rsi,
            "RDI",
            self.rdi,
            "RBP",
            self.rbp,
            "R8",
            self.r8
        );

        log::error!(
            "{:>4}  {:#018X}   {:>4}  {:#018X}   {:>4}  {:#018X}  {:>4}  {:#018X}",
            "R9",
            self.r9,
            "R10",
            self.r10,
            "R11",
            self.r11,
            "R12",
            self.r12
        );

        log::error!(
            "{:>4}  {:#018X}   {:>4}  {:#018X}   {:>4}  {:#018X}",
            "R13",
            self.r13,
            "R14",
            self.r14,
            "R15",
            self.r15
        );

        log::error!("");

        log::debug!("Full Context: {self:#X?}");
    }
}

#[allow(unused)]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nostack));
    }
}

#[allow(unused)]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nostack));
    }
}

#[allow(unused)]
pub fn get_interrupt_state() -> Result<bool, EfiError> {
    let eflags: u64;
    const IF: u64 = 0x200;
    unsafe {
        asm!("pushfq; pop {}", out(reg)eflags);
    }
    Ok(eflags & IF != 0)
}
