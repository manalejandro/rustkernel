// SPDX-License-Identifier: GPL-2.0

//! Kernel main entry point

#![no_std]
#![no_main]

extern crate kernel;

use core::arch::global_asm;

// Include boot assembly
#[cfg(target_arch = "x86_64")]
global_asm!(include_str!("arch/x86_64/boot.s"), options(att_syntax));

/// Entry point called by boot.s assembly code
/// This is just a wrapper to ensure the kernel crate is linked
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
	// This function shouldn't be called directly if boot.s calls kernel_main_multiboot
	// But if it is called, we redirect to the kernel library
	loop {
		unsafe {
			core::arch::asm!("hlt");
		}
	}
}

// Panic handler is defined in the kernel library
