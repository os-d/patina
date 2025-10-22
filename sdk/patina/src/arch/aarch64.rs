//! AArch64 Specific abstractions for Patina.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
//! Portions Copyright 2023 The arm-gic Authors.
//! arm-gic is dual-licensed under Apache 2.0 and MIT terms.

/// Reads and returns the value of the given aarch64 system register.
#[macro_export]
macro_rules! read_sysreg {
    ($name:ident) => {
        {
            let mut value: u64;
            // SAFETY: The caller must provide a valid system register name
            // and ensure that the system is in a state where reading this register is safe.
            unsafe {
                core::arch::asm!(
                    concat!("mrs {value:x}, ", core::stringify!($name)),
                    value = out(reg) value,
                    options(nomem, nostack),
                );
            }
            value
        }
    }
}

/// Writes the given value to the given aarch64 system register.
/// Usage:
/// - `write_sysreg!(reg register_name, imm:value)` - Write immediate value
/// - `write_sysreg!(reg register_name, imm:value, barrier)` - Write immediate value with barrier
/// - `write_sysreg!(reg dest_register, reg src_register)` - Copy from one register to sysreg
/// - `write_sysreg!(reg dest_register, reg src_register, barrier)` - Copy from one register to sysreg with barrier
/// - `write_sysreg!(reg register_name, value)` - Write literal value
/// - `write_sysreg!(reg register_name, value, barrier)` - Write literal value with barrier
#[macro_export]
macro_rules! write_sysreg {
    (reg $dest:ident, imm $imm:literal) => {
        {
            // immediate-to-register copy, no barrier required case
            // SAFETY: The caller must provide valid system register names
            // and ensure that the system is in a state where reading from src and writing to dest is safe.
            unsafe {
                core::arch::asm!(
                    concat!("msr ", core::stringify!($dest), ", {imm}"),
                    imm = const $imm,
                    options(nomem, nostack),
                )
            }
        }
    };
    (reg $dest:ident, imm $imm:literal, $barrier:expr) => {
        {
            // immediate-to-register copy, barrier required case
            // SAFETY: The caller must provide valid system register names
            // and ensure that the system is in a state where reading from src and writing to dest is safe.
            unsafe {
                core::arch::asm!(
                    concat!("msr ", core::stringify!($dest), ", {imm}"),
                    $barrier,
                    imm = const $imm,
                    options(nomem, nostack),
                )
            }
        }
    };
    (reg $dest:ident, reg $src:ident) => {
        {
            // register-to-register copy, no barrier required case
            // SAFETY: The caller must provide valid system register names
            // and ensure that the system is in a state where reading from src and writing to dest is safe.
            unsafe {
                core::arch::asm!(
                    concat!("msr ", core::stringify!($dest), ", ", core::stringify!($src)),
                    options(nomem, nostack),
                )
            }
        }
    };
    (reg $dest:ident, reg $src:ident, $barrier:expr) => {
        {
            // register-to-register copy, barrier required case
            // SAFETY: The caller must provide valid system register names
            // and ensure that the system is in a state where reading from src and writing to dest is safe.
            unsafe {
                core::arch::asm!(
                    concat!("msr ", core::stringify!($dest), ", ", core::stringify!($src)),
                    $barrier,
                    options(nomem, nostack),
                )
            }
        }
    };
    (reg $name:ident, $value:expr) => {
        {
            // no barrier required case
            let v: u64 = $value;
            // SAFETY: The caller must provide a valid system register name
            // and ensure that the system is in a state where writing to this register with this value is safe.
            unsafe {
                core::arch::asm!(
                    concat!("msr ", core::stringify!($name), ", {value:x}"),
                    value = in(reg) v,
                    options(nomem, nostack),
                )
            }
        }
    };
    (reg $name:ident, $value:expr, $barrier:expr) => {
        {
            // barrier required case
            let v: u64 = $value;
            // SAFETY: The caller must provide a valid system register name
            // and ensure that the system is in a state where writing to this register with this value is safe.
            unsafe {
                core::arch::asm!(
                    concat!("msr ", core::stringify!($name), ", {value:x}"),
                    $barrier,
                    value = in(reg) v,
                    options(nomem, nostack),
                )
            }
        }
    };
}
