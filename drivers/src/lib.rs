// SPDX-License-Identifier: GPL-2.0

//! Kernel drivers library
//!
//! This crate contains various kernel drivers for the Rust kernel.

#![no_std]

extern crate alloc;

pub mod dummy;
pub mod mem;
pub mod platform_example;
pub mod ramdisk;
pub mod keyboard; // New keyboard driver
pub mod serial;   // New serial driver

pub use dummy::*;
pub use mem::*;
pub use platform_example::*;
pub use ramdisk::*;
