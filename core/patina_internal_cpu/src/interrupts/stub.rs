//! Stub Interrupt module for tests.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use patina::{error::EfiError, pi::protocols::cpu_arch::EfiSystemContext};

use crate::interrupts::InterruptManager;

/// Null implementation of the EfiSystemContextFactory and EfiExceptionStackTrace traits.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ExceptionContextStub;

impl super::EfiSystemContextFactory for ExceptionContextStub {
    fn create_efi_system_context(&mut self) -> EfiSystemContext {
        // Pointer being set is arbitrary, but EBC is architecture agnostic.
        EfiSystemContext { system_context_ebc: core::ptr::null_mut() }
    }
}

impl super::EfiExceptionStackTrace for ExceptionContextStub {
    fn dump_stack_trace(&self) {}
    fn dump_system_context_registers(&self) {}
}

/// A function that does nothing as this is a null implementation.
#[allow(unused)]
pub fn enable_interrupts() {}

/// A function that does nothing as this is a null implementation.
#[allow(unused)]
pub fn disable_interrupts() {}

/// A function that always returns `true` as this is a null implementation.
#[allow(unused)]
pub fn get_interrupt_state() -> bool {
    true
}

/// Null Implementation of the InterruptManager.
#[derive(Default, Copy, Clone)]
pub struct InterruptsStub {}

impl InterruptsStub {
    /// Creates a new instance of the null implementation of the InterruptManager.
    pub const fn new() -> Self {
        Self {}
    }

    /// A do-nothing initialization function for the null implementation.
    pub fn initialize(&mut self) -> Result<(), EfiError> {
        Ok(())
    }
}

impl InterruptManager for InterruptsStub {}
