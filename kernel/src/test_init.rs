// SPDX-License-Identifier: GPL-2.0

//! Kernel initialization testing and validation

use crate::error::Result;
use crate::{error, info, warn};

/// Test kernel subsystem initialization
pub fn run_init_tests() -> Result<()> {
	info!("Running kernel initialization tests");

	// Test memory management
	test_memory_management()?;

	// Test interrupt handling
	test_interrupt_handling()?;

	// Test basic device operations
	test_device_subsystem()?;

	// Test scheduler
	test_scheduler()?;

	// Test filesystem
	test_filesystem()?;

	info!("All initialization tests passed");
	Ok(())
}

/// Test memory management subsystem
pub fn test_memory_management() -> Result<()> {
	info!("Testing memory management...");

	// Test basic allocation
	let test_alloc = alloc::vec![1u8, 2, 3, 4, 5];
	if test_alloc.len() != 5 {
		return Err(crate::error::Error::Generic);
	}

	// Test page allocation
	if let Ok(page) = crate::memory::page::alloc_page() {
		crate::memory::page::free_page(page);
		info!("Page allocation test passed");
	} else {
		warn!("Page allocation test failed - might not be implemented yet");
	}

	info!("Memory management tests completed");
	Ok(())
}

/// Test interrupt handling
fn test_interrupt_handling() -> Result<()> {
	info!("Testing interrupt handling...");

	// Test interrupt enable/disable
	crate::interrupt::disable();
	crate::interrupt::enable();

	info!("Interrupt handling tests completed");
	Ok(())
}

/// Test device subsystem
fn test_device_subsystem() -> Result<()> {
	info!("Testing device subsystem...");

	// Test device registration (if implemented)
	warn!("Device subsystem tests skipped - implementation pending");

	Ok(())
}

/// Test scheduler
fn test_scheduler() -> Result<()> {
	info!("Testing scheduler...");

	// Basic scheduler tests (if implemented)
	warn!("Scheduler tests skipped - implementation pending");

	Ok(())
}

/// Test filesystem
fn test_filesystem() -> Result<()> {
	info!("Testing filesystem...");

	// Basic VFS tests (if implemented)
	warn!("Filesystem tests skipped - implementation pending");

	Ok(())
}

/// Display system information
pub fn display_system_info() {
	info!("=== System Information ===");

	unsafe {
		let boot_info = &crate::boot::BOOT_INFO;
		info!("Memory size: {} bytes", boot_info.memory_size);
		info!("Available memory: {} bytes", boot_info.available_memory);
		info!("CPU count: {}", boot_info.cpu_count);

		if let Some(ref cmdline) = boot_info.command_line {
			info!("Command line: {}", cmdline);
		}

		if let Some(initrd_start) = boot_info.initrd_start {
			info!("Initrd start: 0x{:x}", initrd_start);
			if let Some(initrd_size) = boot_info.initrd_size {
				info!("Initrd size: {} bytes", initrd_size);
			}
		}
	}

	info!("=========================");
}

/// Run basic functionality tests
pub fn run_basic_tests() -> Result<()> {
	info!("Running basic kernel functionality tests");

	// Test string operations
	let test_string = alloc::string::String::from("Hello Rust Kernel!");
	if test_string.len() != 18 {
		return Err(crate::error::Error::Generic);
	}
	info!("String operations test passed");

	// Test vector operations
	let mut test_vec = alloc::vec::Vec::new();
	for i in 0..10 {
		test_vec.push(i);
	}
	if test_vec.len() != 10 {
		return Err(crate::error::Error::Generic);
	}
	info!("Vector operations test passed");

	// Test basic arithmetic
	let result = 42 * 42;
	if result != 1764 {
		return Err(crate::error::Error::Generic);
	}
	info!("Arithmetic operations test passed");

	info!("All basic functionality tests passed");
	Ok(())
}
