// SPDX-License-Identifier: GPL-2.0

//! The Rust kernel crate.
//!
//! This crate provides the core kernel APIs and functionality for the Rust kernel.
//! It is inspired by the Linux kernel's Rust infrastructure but designed as a
//! standalone kernel implementation.

#![no_std]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod allocator;
pub mod arch;
pub mod boot;
pub mod console;
pub mod cpu;
pub mod device;
pub mod driver;
pub mod error;
pub mod fs;
pub mod init;
pub mod interrupt;
pub mod memory;
pub mod module;
pub mod panic;
pub mod prelude;
pub mod process;
pub mod scheduler;
pub mod sync;
pub mod syscall;
pub mod task;
pub mod time;
pub mod types;

use core::panic::PanicInfo;

/// Kernel version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "Rust Kernel";

/// Kernel entry point called from architecture-specific code
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    init::early_init();
    init::main_init();
    
    // Should not return from main_init
    panic!("kernel_main returned unexpectedly");
}

/// Test runner for kernel tests
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

#[cfg(test)]
pub fn exit_qemu(exit_code: QemuExitCode) {
    use arch::x86_64::port::Port;
    
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Kernel panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}

/// Global allocator error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
