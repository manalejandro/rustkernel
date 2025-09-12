// SPDX-License-Identifier: GPL-2.0

//! Kernel drivers library
//!
//! This crate contains various kernel drivers for the Rust kernel.

#![no_std]

extern crate alloc;

pub mod keyboard; // Keyboard driver
pub mod mem;
pub mod ramdisk;
pub mod rtl8139;
pub mod serial; // Serial driver
pub use ramdisk::*;
