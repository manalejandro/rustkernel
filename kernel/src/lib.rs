// SPDX-License-Identifier: GPL-2.0

//! The Rust kernel crate.
//!
//! This crate provides the core kernel APIs and functionality for the Rust
//! kernel. It is inspired by the Linux kernel's Rust infrastructure but
//! designed as a standalone kernel implementation.

#![no_std]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![feature(allocator_api)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

// Include boot assembly
// #[cfg(target_arch = "x86_64")]
// global_asm!(include_str!("arch/x86_64/boot.s"), options(att_syntax));

pub mod advanced_perf; // Advanced performance monitoring and profiling
pub mod arch;
pub mod arp;
pub mod benchmark; // Performance benchmarking
pub mod boot;
pub mod console;
pub mod cpu;
pub mod device;
pub mod device_advanced;
pub mod diagnostics; // System diagnostics and health monitoring
pub mod driver;
pub mod drivers_init; // Driver initialization
pub mod enhanced_scheduler; // Enhanced preemptive scheduler
pub mod error;
pub mod fs;
pub mod hardware; // Hardware detection and initialization
pub mod icmp;
pub mod init;
pub mod interrupt;
pub mod ipc; // Inter-process communication
pub mod kthread; // Kernel thread management
pub mod logging; // Kernel logging and debugging
pub mod memfs; // In-memory file system
pub mod memory;
pub mod module;
pub mod module_loader; // Dynamic module loading
pub mod network;
pub mod panic;
pub mod perf; // Performance monitoring
pub mod prelude;
pub mod process;
pub mod scheduler;
pub mod shell; // Kernel shell interface
pub mod stress_test; // System stress testing
pub mod sync;
pub mod syscall;
pub mod syscalls; // New syscall infrastructure
pub mod sysinfo; // System information and hardware detection
pub mod task;
pub mod test_init; // Kernel initialization testing
pub mod test_suite; // Comprehensive kernel test suite
pub mod time;
pub mod timer; // Timer interrupt and preemptive scheduling
pub mod types;
pub mod usermode;
pub mod working_task; // Working kernel task implementation // User mode program support

/// Kernel version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "Rust Kernel";

/// Kernel entry point called from architecture-specific code
/// This is called from the boot assembly with multiboot information
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
	// Early initialization without memory allocation
	early_kernel_init();

	// Initialize memory management
	if let Err(e) = memory_init() {
		panic!("Memory initialization failed: {:?}", e);
	}

	// Now we can use allocations, continue with full initialization
	init::early_init();

	init::main_init();

	// Should not return from main_init
	panic!("kernel_main returned unexpectedly");
}

/// Kernel entry point with multiboot parameters
#[no_mangle]
pub extern "C" fn kernel_main_multiboot(multiboot_magic: u32, multiboot_addr: u32) -> ! {
	// Verify multiboot magic number
	if multiboot_magic != 0x36d76289 && multiboot_magic != 0x2BADB002 {
		panic!("Invalid multiboot magic: 0x{:x}", multiboot_magic);
	}

	// Store multiboot information
	boot::set_multiboot_info(multiboot_addr as usize);

	// Continue with normal boot
	kernel_main();
}

/// Early kernel initialization before memory allocator is available
fn early_kernel_init() {
	// Initialize console first so we can print messages
	if let Err(_) = console::init() {
		// Can't print error since console isn't initialized
		loop {}
	}

	crate::console::write_str("\n");
	crate::console::write_str("Booting Rust Kernel...\n");
}

/// Initialize memory management using multiboot information
fn memory_init() -> Result<(), error::Error> {
	crate::console::write_str("[*] Initializing memory subsystem...\n");

	// FIXME: Multiboot parsing causes crashes - use default memory layout for now
	memory::page::init()?;

	// Initialize heap allocator
	memory::kmalloc::init()?;

	crate::console::write_str("[+] Memory subsystem ready\n");
	Ok(())
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

/// Global allocator error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
	panic!("allocation error: {:?}", layout)
}
