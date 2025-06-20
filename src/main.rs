// SPDX-License-Identifier: GPL-2.0

//! Rust Kernel Binary
//!
//! Main binary entry point for the Rust kernel

#![no_std]
#![no_main]

extern crate kernel;

/// Main kernel entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel::kernel_main()
}
