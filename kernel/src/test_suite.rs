// SPDX-License-Identifier: GPL-2.0

//! Comprehensive kernel test suite

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::error::{Error, Result};

/// Test result structure
#[derive(Debug, Clone)]
pub struct TestResult {
	pub name: String,
	pub passed: bool,
	pub message: String,
	pub duration_ms: u64,
}

/// Test suite statistics
#[derive(Debug, Clone)]
pub struct TestStats {
	pub total_tests: u32,
	pub passed_tests: u32,
	pub failed_tests: u32,
	pub total_duration_ms: u64,
}

/// Run all kernel tests
pub fn run_all_tests() -> Result<TestStats> {
	crate::info!("Starting comprehensive kernel test suite...");

	let mut results = Vec::new();
	let start_time = crate::time::get_time_ns();

	// Memory management tests
	results.extend(test_memory_management()?);

	// Scheduler tests
	results.extend(test_scheduler()?);

	// IPC tests
	results.extend(test_ipc()?);

	// Performance monitoring tests
	results.extend(test_performance_monitoring()?);

	// File system tests
	results.extend(test_filesystem()?);

	// Hardware detection tests
	results.extend(test_hardware_detection()?);

	// Timer tests
	results.extend(test_timer_functionality()?);

	let end_time = crate::time::get_time_ns();
	let total_duration = (end_time - start_time) / 1_000_000; // Convert to ms

	// Calculate statistics
	let stats = TestStats {
		total_tests: results.len() as u32,
		passed_tests: results.iter().filter(|r| r.passed).count() as u32,
		failed_tests: results.iter().filter(|r| !r.passed).count() as u32,
		total_duration_ms: total_duration,
	};

	// Print results summary
	print_test_summary(&results, &stats);

	Ok(stats)
}

/// Test memory management functionality
fn test_memory_management() -> Result<Vec<TestResult>> {
	let mut results = Vec::new();

	// Test basic allocation
	results.push(test_basic_allocation());

	// Test advanced allocator stats
	results.push(test_allocator_stats());

	// Test heap operations
	results.push(test_heap_operations());

	Ok(results)
}

/// Test basic memory allocation
fn test_basic_allocation() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		// Test kmalloc
		let ptr = crate::memory::kmalloc::kmalloc(1024)?;
		if ptr.is_null() {
			return Err(crate::error::Error::ENOMEM);
		}

		// Test writing to allocated memory
		unsafe {
			core::ptr::write(ptr, 42u8);
			let value = core::ptr::read(ptr);
			if value != 42 {
				return Err(crate::error::Error::EIO);
			}
		}

		// Free memory
		crate::memory::kmalloc::kfree(ptr);

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Basic Memory Allocation".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test allocator statistics
fn test_allocator_stats() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let stats = crate::memory::advanced_allocator::get_memory_stats();

		// Basic sanity checks
		if stats.allocation_count < stats.active_allocations as u64 {
			return Err(crate::error::Error::EIO);
		}

		if stats.peak_usage < stats.current_allocated {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Allocator Statistics".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Stats validation failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test heap operations
fn test_heap_operations() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let initial_heap = crate::memory::get_heap_end();
		let new_heap = crate::types::VirtAddr::new(initial_heap.as_usize() + 4096);

		// Test heap expansion
		crate::memory::set_heap_end(new_heap)?;

		let current_heap = crate::memory::get_heap_end();
		if current_heap != new_heap {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Heap Operations".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Heap operations failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test scheduler functionality
fn test_scheduler() -> Result<Vec<TestResult>> {
	let mut results = Vec::new();

	results.push(test_scheduler_stats());
	results.push(test_task_creation());

	Ok(results)
}

/// Test scheduler statistics
fn test_scheduler_stats() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let stats = crate::enhanced_scheduler::get_scheduler_stats();

		// Basic validation
		if stats.total_tasks < stats.runnable_tasks {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Scheduler Statistics".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Scheduler stats invalid".to_string()
		},
		duration_ms: duration,
	}
}

/// Test task creation
fn test_task_creation() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let initial_count = crate::working_task::get_task_count();

		// Create a test task
		let _task_id =
			crate::working_task::create_kernel_task("test_task", test_task_function)?;

		let new_count = crate::working_task::get_task_count();
		if new_count <= initial_count {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Task Creation".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Task creation failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test IPC functionality
fn test_ipc() -> Result<Vec<TestResult>> {
	let mut results = Vec::new();

	results.push(test_ipc_stats());
	results.push(test_message_queue());

	Ok(results)
}

/// Test IPC statistics
fn test_ipc_stats() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let stats = crate::ipc::get_ipc_stats();

		// Basic validation - stats should be consistent
		if stats.messages_sent < stats.messages_received && stats.messages_received > 0 {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "IPC Statistics".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"IPC stats invalid".to_string()
		},
		duration_ms: duration,
	}
}

/// Test message queue operations
fn test_message_queue() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		// Create a message queue (simplified test)
		let test_tid = crate::types::Tid(1);
		let _queue_result = crate::ipc::create_message_queue(test_tid, 1024);

		// Send a test message
		let test_data = b"Hello, IPC!";
		let sender_tid = crate::types::Tid(1);
		let recipient_tid = crate::types::Tid(2);
		let message_type = crate::ipc::MessageType::Data;
		let data_vec = test_data.to_vec();
		let _send_result = crate::ipc::send_message(
			sender_tid,
			recipient_tid,
			message_type,
			data_vec,
			1,
		);

		// Try to receive the message
		if let Ok(Some(_message)) = crate::ipc::receive_message(test_tid) {
			Ok(())
		} else {
			Err(crate::error::Error::EIO)
		}
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Message Queue Operations".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Message queue test failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test performance monitoring
fn test_performance_monitoring() -> Result<Vec<TestResult>> {
	let mut results = Vec::new();

	results.push(test_perf_counters());
	results.push(test_profiling());

	Ok(results)
}

/// Test performance counters
fn test_perf_counters() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let summary = crate::advanced_perf::get_performance_summary();

		// Check if monitoring is enabled
		if !summary.monitoring_enabled {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Performance Counters".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Performance monitoring disabled".to_string()
		},
		duration_ms: duration,
	}
}

/// Test profiling functionality
fn test_profiling() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		// Start profiling for a test function
		let _guard = crate::advanced_perf::profile_function("test_function");

		// Do some work
		for _i in 0..1000 {
			unsafe { core::arch::asm!("nop") };
		}

		// Guard should automatically stop profiling when dropped
		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Function Profiling".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Profiling failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test file system functionality
fn test_filesystem() -> Result<Vec<TestResult>> {
	let mut results = Vec::new();

	results.push(test_fs_basic_ops());

	Ok(results)
}

/// Test basic file system operations
fn test_fs_basic_ops() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		// Test VFS initialization
		let _vfs = crate::fs::get_root_fs()?;

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "File System Basic Operations".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"VFS operations failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test hardware detection
fn test_hardware_detection() -> Result<Vec<TestResult>> {
	let mut results = Vec::new();

	results.push(test_cpu_detection());
	results.push(test_memory_detection());

	Ok(results)
}

/// Test CPU detection
fn test_cpu_detection() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let cpu_info = crate::hardware::detect_cpu()?;

		if cpu_info.vendor.is_empty() {
			return Err(crate::error::Error::EIO);
		}

		if cpu_info.core_count == 0 {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "CPU Detection".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"CPU detection failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test memory detection
fn test_memory_detection() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let memory_size = crate::hardware::detect_memory()?;

		if memory_size < 16 * 1024 * 1024 {
			// Less than 16MB seems wrong
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Memory Detection".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Memory detection failed".to_string()
		},
		duration_ms: duration,
	}
}

/// Test timer functionality
fn test_timer_functionality() -> Result<Vec<TestResult>> {
	let mut results = Vec::new();

	results.push(test_timer_basic());
	results.push(test_jiffies());

	Ok(results)
}

/// Test basic timer functionality
fn test_timer_basic() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let time1 = crate::time::get_time_ns();

		// Do some work
		for _i in 0..100 {
			unsafe { core::arch::asm!("nop") };
		}

		let time2 = crate::time::get_time_ns();

		if time2 <= time1 {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Timer Basic Functionality".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Timer not working".to_string()
		},
		duration_ms: duration,
	}
}

/// Test jiffies counter
fn test_jiffies() -> TestResult {
	let start = crate::time::get_time_ns();

	let result = || -> Result<()> {
		let jiffies1 = crate::time::get_jiffies();

		// Wait a bit (simulate time passing)
		for _i in 0..1000 {
			unsafe { core::arch::asm!("nop") };
		}

		let jiffies2 = crate::time::get_jiffies();

		// Jiffies should either be the same or have incremented
		if jiffies2.0 < jiffies1.0 {
			return Err(crate::error::Error::EIO);
		}

		Ok(())
	}();

	let end = crate::time::get_time_ns();
	let duration = (end - start) / 1_000_000;

	TestResult {
		name: "Jiffies Counter".to_string(),
		passed: result.is_ok(),
		message: if result.is_ok() {
			"Passed".to_string()
		} else {
			"Jiffies counter broken".to_string()
		},
		duration_ms: duration,
	}
}

/// Test task function for task creation test
fn test_task_function() {
	// Simple test task that does nothing
	crate::info!("Test task executing");
}

/// Print test results summary
fn print_test_summary(results: &[TestResult], stats: &TestStats) {
	crate::info!("=== KERNEL TEST SUITE RESULTS ===");
	crate::info!("Total tests: {}", stats.total_tests);
	crate::info!("Passed: {}", stats.passed_tests);
	crate::info!("Failed: {}", stats.failed_tests);
	crate::info!(
		"Success rate: {:.1}%",
		(stats.passed_tests as f32 / stats.total_tests as f32) * 100.0
	);
	crate::info!("Total duration: {} ms", stats.total_duration_ms);

	if stats.failed_tests > 0 {
		crate::info!("Failed tests:");
		for result in results {
			if !result.passed {
				crate::info!(
					"  - {} ({}ms): {}",
					result.name,
					result.duration_ms,
					result.message
				);
			}
		}
	}

	crate::info!("=== END TEST RESULTS ===");
}

/// Initialize test suite
pub fn init() -> Result<()> {
	crate::info!("Kernel test suite initialized");
	Ok(())
}
