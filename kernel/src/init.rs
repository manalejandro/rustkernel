// SPDX-License-Identifier: GPL-2.0

//! Kernel initialization

use crate::error::Result;

/// Early kernel initialization
pub fn early_init() {
	crate::console::write_str("[+] Early initialization complete\n");
}

/// Initialize all kernel subsystems
fn init_subsystems() {
	crate::console::write_str("[*] Initializing kernel subsystems...\n");

	// Initialize timer system
	crate::console::write_str("    - Timer system\n");
	if let Err(_e) = crate::timer::init_timer() {
		crate::console::write_str("      [!] Timer init failed (non-fatal)\n");
	}

	// Initialize interrupt handlers
	crate::console::write_str("    - Interrupt handlers\n");
	if let Err(_e) = crate::interrupt::init() {
		crate::console::write_str("      [!] Interrupt init failed (non-fatal)\n");
	}

	// Initialize scheduler
	crate::console::write_str("    - Scheduler\n");
	if let Err(_e) = crate::enhanced_scheduler::init_enhanced_scheduler() {
		crate::console::write_str("      [!] Scheduler init failed (non-fatal)\n");
	}

	// Initialize IPC subsystem
	crate::console::write_str("    - IPC subsystem\n");
	if let Err(_e) = crate::ipc::init_ipc() {
		crate::console::write_str("      [!] IPC init failed (non-fatal)\n");
	}

	// Initialize performance monitoring
	crate::console::write_str("    - Performance monitoring\n");
	if let Err(_e) = crate::advanced_perf::init_performance_monitoring() {
		crate::console::write_str("      [!] Perf init failed (non-fatal)\n");
	}

	// Initialize diagnostics
	crate::console::write_str("    - System diagnostics\n");
	if let Err(_e) = crate::diagnostics::init_diagnostics() {
		crate::console::write_str("      [!] Diagnostics init failed (non-fatal)\n");
	}

	// Initialize working task manager
	crate::console::write_str("    - Task manager\n");
	if let Err(_e) = crate::working_task::init_task_management() {
		crate::console::write_str("      [!] Task mgmt init failed (non-fatal)\n");
	}

	crate::console::write_str("[+] Subsystems initialized\n");
}

/// Main kernel initialization  
pub fn main_init() -> ! {
	// Print boot banner
	crate::console::write_str("\n");
	crate::console::write_str("========================================\n");
	crate::console::write_str("         Rust Kernel v0.1.0\n");
	crate::console::write_str("========================================\n");
	crate::console::write_str("\n");

	// Initialize subsystems
	init_subsystems();

	// Print system information
	crate::console::write_str("\n");
	crate::console::write_str("System Information:\n");
	crate::console::write_str("  Architecture: x86_64\n");
	crate::console::write_str("  Memory mapping: 0-1GB identity mapped\n");
	crate::console::write_str("  Page size: 2MB (large pages)\n");
	crate::console::write_str("\n");
	crate::console::write_str("[+] Kernel initialization complete\n");
	crate::console::write_str("\n");

	// Enter main kernel loop
	main_kernel_loop()
}

/// Main kernel loop with task scheduling
fn main_kernel_loop() -> ! {
	crate::console::write_str("Entering kernel main loop...\n");

	let mut tick_count: u64 = 0;

	loop {
		tick_count = tick_count.wrapping_add(1);

		// Handle timer tick periodically
		if tick_count % 10000 == 0 {
			crate::timer::handle_timer_tick();
		}

		// Schedule next task
		if let Some(_tid) = crate::enhanced_scheduler::schedule_next() {
			// Task would be executed here
			for _ in 0..100 {
				unsafe { core::arch::asm!("pause"); }
			}
		}

		// Cleanup terminated tasks periodically
		if tick_count % 100000 == 0 {
			crate::working_task::cleanup_tasks();
		}

		// Heartbeat indicator
		if tick_count % 5_000_000 == 0 {
			crate::console::write_str(".");
		}

		// Halt CPU to save power
		unsafe { core::arch::asm!("hlt"); }
	}
}
