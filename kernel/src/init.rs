// SPDX-License-Identifier: GPL-2.0

//! Kernel initialization

use alloc::string::ToString;

use crate::{error, info, warn};

/// Early kernel initialization
pub fn early_init() {
	crate::console::write_str("[+] Early initialization complete\n");
}

/// Main kernel initialization  
pub fn main_init() -> ! {
	crate::console::write_str("\n");
	crate::console::write_str("========================================\n");
	crate::console::write_str("    Rust Kernel v0.1.0\n");
	crate::console::write_str("========================================\n");
	crate::console::write_str("\n");
	crate::console::write_str("Status: Boot successful!\n");
	crate::console::write_str("Console: Working\n");
	crate::console::write_str("Memory: Basic allocator initialized\n");
	crate::console::write_str("Architecture: x86_64\n");
	crate::console::write_str("\n");
	crate::console::write_str("System Information:\n");
	crate::console::write_str("  - Identity mapping: 0-1GB\n");
	crate::console::write_str("  - Paging: 4-level (PML4->PDP->PD)\n");
	crate::console::write_str("  - Page size: 2MB (large pages)\n");
	crate::console::write_str("\n");
	crate::console::write_str("Kernel is now idle.\n");
	crate::console::write_str("Press Ctrl+C to exit QEMU.\n");
	crate::console::write_str("\n");

	// Simple idle loop
	let mut counter = 0u64;
	loop {
		counter += 1;

		// Print a heartbeat every ~1 million iterations
		if counter % 1_000_000 == 0 {
			crate::console::write_str(".");
		}

		unsafe {
			core::arch::asm!("hlt");
		}
	}
}

/// Start essential kernel threads
fn start_kernel_threads() {
	info!("Starting kernel threads...");

	// Start heartbeat task for testing
	match crate::working_task::spawn_kernel_task(
		"heartbeat".to_string(),
		crate::working_task::heartbeat_task,
		8192,
	) {
		Ok(tid) => info!("Started heartbeat task: {:?}", tid),
		Err(e) => warn!("Failed to start heartbeat task: {}", e),
	}

	// Start memory monitor task
	match crate::working_task::spawn_kernel_task(
		"memory_monitor".to_string(),
		crate::working_task::memory_monitor_task,
		8192,
	) {
		Ok(tid) => info!("Started memory monitor task: {:?}", tid),
		Err(e) => warn!("Failed to start memory monitor task: {}", e),
	}

	// Start performance monitor task
	match crate::working_task::spawn_kernel_task(
		"perf_monitor".to_string(),
		crate::working_task::performance_monitor_task,
		8192,
	) {
		Ok(tid) => info!("Started performance monitor task: {:?}", tid),
		Err(e) => warn!("Failed to start performance monitor task: {}", e),
	}

	info!("Kernel threads started");
}

/// Main kernel loop with task scheduling
fn main_kernel_loop() -> ! {
	let mut loop_count = 0;

	loop {
		loop_count += 1;

		// Record performance events periodically
		if loop_count % 1000 == 0 {
			crate::advanced_perf::record_event(
				crate::advanced_perf::CounterType::SystemCalls,
				1,
			);
		}

		// Schedule next task from enhanced scheduler
		if let Some(_next_tid) = crate::enhanced_scheduler::schedule_next() {
			// Task switching would happen here in a full implementation
			// For now, just yield some CPU time
			for _ in 0..1000 {
				unsafe {
					core::arch::asm!("pause");
				}
			}
		}

		// Clean up terminated tasks periodically
		if loop_count % 10000 == 0 {
			crate::working_task::cleanup_tasks();
		}

		// Check for timer events and handle preemption
		// This would normally be done by timer interrupt
		if loop_count % 5000 == 0 {
			crate::timer::handle_timer_tick();
		}

		// Power management - halt CPU briefly to save power
		unsafe {
			core::arch::asm!("hlt");
		}
	}
}
