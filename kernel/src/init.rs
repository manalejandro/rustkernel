// SPDX-License-Identifier: GPL-2.0

//! Kernel initialization

use alloc::string::ToString;

use crate::{error, info, warn};

/// Early kernel initialization
pub fn early_init() {
	info!("Starting Rust Kernel v{}", crate::VERSION);
	info!("Early initialization phase");

	// Initialize basic console output
	if let Err(e) = crate::console::init() {
		// Can't print error since console isn't initialized yet
		loop {}
	}

	info!("Console initialized");
}

/// Main kernel initialization  
pub fn main_init() -> ! {
	info!("Main initialization phase");

	// Initialize memory management
	if let Err(e) = crate::memory::init() {
		error!("Failed to initialize memory management: {}", e);
		panic!("Memory initialization failed");
	}
	info!("Memory management initialized");

	// Hardware detection and initialization
	if let Err(e) = crate::hardware::init() {
		error!("Failed to initialize hardware detection: {}", e);
		panic!("Hardware detection failed");
	}
	info!("Hardware detection completed");

	// Initialize heap for brk syscall
	if let Err(e) = crate::memory::init_heap() {
		error!("Failed to initialize heap: {}", e);
		panic!("Heap initialization failed");
	}
	info!("Heap initialized");

	// Initialize kmalloc
	if let Err(e) = crate::memory::kmalloc::init() {
		error!("Failed to initialize kmalloc: {}", e);
		panic!("Kmalloc initialization failed");
	}
	info!("Kmalloc initialized");

	// Initialize vmalloc
	if let Err(e) = crate::memory::vmalloc::init() {
		error!("Failed to initialize vmalloc: {}", e);
		panic!("Vmalloc initialization failed");
	}
	info!("Vmalloc initialized");

	// Initialize interrupt handling
	if let Err(e) = crate::interrupt::init() {
		error!("Failed to initialize interrupts: {}", e);
		panic!("Interrupt initialization failed");
	}
	info!("Interrupt handling initialized");

	// Initialize scheduler
	if let Err(e) = crate::scheduler::init() {
		error!("Failed to initialize scheduler: {}", e);
		panic!("Scheduler initialization failed");
	}
	info!("Scheduler initialized");

	// Initialize enhanced scheduler
	if let Err(e) = crate::enhanced_scheduler::init_enhanced_scheduler() {
		error!("Failed to initialize enhanced scheduler: {}", e);
		panic!("Enhanced scheduler initialization failed");
	}
	info!("Enhanced scheduler initialized");

	// Initialize IPC system
	if let Err(e) = crate::ipc::init_ipc() {
		error!("Failed to initialize IPC system: {}", e);
		panic!("IPC system initialization failed");
	}
	info!("IPC system initialized");

	// Initialize advanced performance monitoring
	if let Err(e) = crate::advanced_perf::init_performance_monitoring() {
		error!(
			"Failed to initialize advanced performance monitoring: {}",
			e
		);
		panic!("Advanced performance monitoring initialization failed");
	}
	info!("Advanced performance monitoring initialized");

	// Initialize timer for preemptive scheduling
	if let Err(e) = crate::timer::init_timer() {
		error!("Failed to initialize timer: {}", e);
		panic!("Timer initialization failed");
	}
	info!("Timer initialized");

	// Initialize device subsystem
	if let Err(e) = crate::device::init() {
		error!("Failed to initialize devices: {}", e);
		panic!("Device initialization failed");
	}
	info!("Device subsystem initialized");

	// Initialize VFS (Virtual File System)
	if let Err(e) = crate::fs::init() {
		error!("Failed to initialize VFS: {}", e);
		panic!("VFS initialization failed");
	}
	info!("VFS initialized");

	// Initialize process management
	if let Err(e) = crate::process::init_process_management() {
		error!("Failed to initialize process management: {}", e);
		panic!("Process management initialization failed");
	}
	info!("Process management initialized");

	// Initialize system calls
	if let Err(e) = crate::syscalls::init_syscalls() {
		error!("Failed to initialize syscalls: {}", e);
		panic!("Syscall initialization failed");
	}
	info!("System calls initialized");

	// Initialize time management
	if let Err(e) = crate::time::init_time() {
		error!("Failed to initialize time management: {}", e);
		panic!("Time management initialization failed");
	}
	info!("Time management initialized");

	// Display system information
	crate::test_init::display_system_info();

	// Run basic functionality tests
	if let Err(e) = crate::test_init::run_basic_tests() {
		error!("Basic functionality tests failed: {}", e);
		panic!("Basic tests failed");
	}

	// Run initialization tests
	if let Err(e) = crate::test_init::run_init_tests() {
		error!("Initialization tests failed: {}", e);
		panic!("Initialization tests failed");
	}

	// Initialize drivers
	if let Err(e) = crate::drivers_init::init_drivers() {
		error!("Failed to initialize drivers: {}", e);
		panic!("Driver initialization failed");
	}
	info!("Drivers initialized");

	// Initialize kernel threads
	if let Err(e) = crate::kthread::init_kthreads() {
		error!("Failed to initialize kernel threads: {}", e);
		panic!("Kernel thread initialization failed");
	}
	info!("Kernel threads initialized");

	// Initialize kernel shell
	if let Err(e) = crate::shell::init_shell() {
		error!("Failed to initialize kernel shell: {}", e);
		panic!("Shell initialization failed");
	}
	info!("Kernel shell initialized");

	// Initialize networking
	if let Err(e) = crate::network::init() {
		error!("Failed to initialize networking: {}", e);
		panic!("Networking initialization failed");
	}
	info!("Networking initialized");

	// Initialize module loading system
	if let Err(e) = crate::module_loader::init_modules() {
		error!("Failed to initialize module system: {}", e);
		panic!("Module system initialization failed");
	}
	info!("Module system initialized");

	// Initialize in-memory file system
	if let Err(e) = crate::memfs::init_memfs() {
		error!("Failed to initialize file system: {}", e);
		panic!("File system initialization failed");
	}
	info!("In-memory file system initialized");

	// Initialize test suite
	if let Err(e) = crate::test_suite::init() {
		error!("Failed to initialize test suite: {}", e);
		panic!("Test suite initialization failed");
	}
	info!("Test suite initialized");

	// Initialize user mode support
	if let Err(e) = crate::usermode::init_usermode() {
		error!("Failed to initialize user mode: {}", e);
		panic!("User mode initialization failed");
	}
	info!("User mode support initialized");

	// Initialize performance monitoring
	if let Err(e) = crate::perf::init_perf_monitor() {
		error!("Failed to initialize performance monitoring: {}", e);
		panic!("Performance monitoring initialization failed");
	}
	info!("Performance monitoring initialized");

	// Initialize advanced logging system
	if let Err(e) = crate::logging::init_logging() {
		error!("Failed to initialize logging system: {}", e);
		panic!("Logging system initialization failed");
	}
	info!("Advanced logging system initialized");

	// Initialize system information collection
	if let Err(e) = crate::sysinfo::init_sysinfo() {
		error!("Failed to initialize system information: {}", e);
		panic!("System information initialization failed");
	}
	info!("System information collection initialized");

	// Initialize system diagnostics
	if let Err(e) = crate::diagnostics::init_diagnostics() {
		error!("Failed to initialize system diagnostics: {}", e);
		panic!("System diagnostics initialization failed");
	}
	info!("System diagnostics initialized");

	// Initialize working task management
	if let Err(e) = crate::working_task::init_task_management() {
		error!("Failed to initialize task management: {}", e);
		panic!("Task management initialization failed");
	}
	info!("Task management initialized");

	// Start kernel threads
	start_kernel_threads();

	info!("Kernel initialization completed");
	info!("Starting main kernel loop");

	// Start the main kernel loop
	main_kernel_loop();
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
