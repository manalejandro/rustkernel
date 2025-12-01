// SPDX-License-Identifier: GPL-2.0

//! Kernel main entry point

#![no_std]
#![no_main]

extern crate kernel;

use core::arch::global_asm;

// Include boot assembly
#[cfg(target_arch = "x86_64")]
global_asm!(include_str!("arch/x86_64/boot.s"), options(att_syntax));

use core::panic::PanicInfo;

/// Multiboot1 header - placed at the very beginning
#[repr(C)]
#[repr(packed)]
struct MultibootHeader {
	magic: u32,
	flags: u32,
	checksum: u32,
}

/// Multiboot header must be in the first 8KB and be 4-byte aligned
// #[link_section = ".multiboot_header"]
// #[no_mangle]
// #[used]
// static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
// 	magic: 0x1BADB002,
// 	flags: 0x00000000,
// 	checksum: 0u32.wrapping_sub(0x1BADB002u32.wrapping_add(0x00000000)),
// };

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

#[no_mangle]
pub extern "C" fn _start() -> ! {
	loop {}
}
