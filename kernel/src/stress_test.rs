// SPDX-License-Identifier: GPL-2.0

//! System stress testing and load generation

use alloc::{format, string::String, vec::Vec};

use crate::error::Result;
use crate::time::get_jiffies;
use crate::types::Jiffies;

/// Stress test types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StressTestType {
	Memory,
	CPU,
	IO,
	FileSystem,
	Network,
	All,
}

/// Stress test results
#[derive(Debug, Clone)]
pub struct StressTestResult {
	pub test_type: StressTestType,
	pub duration_jiffies: u64,
	pub operations_completed: u64,
	pub operations_per_second: u64,
	pub errors_encountered: u64,
	pub details: String,
}

/// Memory stress test - allocate and free memory rapidly
pub fn memory_stress_test(duration_seconds: u64) -> Result<StressTestResult> {
	let start_time = get_jiffies();
	let duration_jiffies = duration_seconds * 1000; // Convert to jiffies (1000 Hz)
	let mut operations = 0u64;
	let mut errors = 0u64;
	let mut allocations: Vec<*mut u8> = Vec::new();

	while (get_jiffies() - start_time).as_u64() < duration_jiffies {
		// Allocate memory
		match crate::memory::kmalloc::kmalloc(1024) {
			Ok(ptr) => {
				allocations.push(ptr);
				operations += 1;

				// Free every 100 allocations to prevent exhaustion
				if allocations.len() >= 100 {
					for ptr in allocations.drain(..) {
						crate::memory::kmalloc::kfree(ptr);
						operations += 1;
					}
				}
			}
			Err(_) => {
				errors += 1;
				// Free all allocations on error
				for ptr in allocations.drain(..) {
					crate::memory::kmalloc::kfree(ptr);
				}
			}
		}
	}

	// Clean up remaining allocations
	for ptr in allocations.drain(..) {
		crate::memory::kmalloc::kfree(ptr);
		operations += 1;
	}

	let actual_duration = (get_jiffies() - start_time).as_u64();
	let ops_per_second = if actual_duration > 0 {
		(operations * 1000) / actual_duration
	} else {
		0
	};

	Ok(StressTestResult {
		test_type: StressTestType::Memory,
		duration_jiffies: actual_duration,
		operations_completed: operations,
		operations_per_second: ops_per_second,
		errors_encountered: errors,
		details: format!("Allocated/freed {} KB total", operations / 2),
	})
}

/// CPU stress test - perform intensive calculations
pub fn cpu_stress_test(duration_seconds: u64) -> Result<StressTestResult> {
	let start_time = get_jiffies();
	let duration_jiffies = duration_seconds * 1000;
	let mut operations = 0u64;
	let mut result = 1u64;

	while (get_jiffies() - start_time).as_u64() < duration_jiffies {
		// Perform some CPU-intensive operations
		for i in 1..1000 {
			result = result.wrapping_mul(i).wrapping_add(i * i);
			operations += 1;
		}

		// Prevent optimization from removing the loop
		if result == 0 {
			break;
		}
	}

	let actual_duration = (get_jiffies() - start_time).as_u64();
	let ops_per_second = if actual_duration > 0 {
		(operations * 1000) / actual_duration
	} else {
		0
	};

	Ok(StressTestResult {
		test_type: StressTestType::CPU,
		duration_jiffies: actual_duration,
		operations_completed: operations,
		operations_per_second: ops_per_second,
		errors_encountered: 0,
		details: format!("Final calculation result: {}", result),
	})
}

/// File system stress test - create, write, read, delete files
pub fn filesystem_stress_test(duration_seconds: u64) -> Result<StressTestResult> {
	let start_time = get_jiffies();
	let duration_jiffies = duration_seconds * 1000;
	let mut operations = 0u64;
	let mut errors = 0u64;
	let mut file_counter = 0u32;

	while (get_jiffies() - start_time).as_u64() < duration_jiffies {
		let filename = format!("/tmp/stress_test_{}", file_counter);
		file_counter += 1;

		// Create file
		match crate::memfs::fs_create_file(&filename) {
			Ok(()) => operations += 1,
			Err(_) => errors += 1,
		}

		// Write to file (not implemented in memfs, but count the attempt)
		operations += 1;

		// Read file (attempt)
		match crate::memfs::fs_read(&filename) {
			Ok(_) => operations += 1,
			Err(_) => errors += 1,
		}

		// Delete file
		match crate::memfs::fs_remove(&filename) {
			Ok(()) => operations += 1,
			Err(_) => errors += 1,
		}
	}

	let actual_duration = (get_jiffies() - start_time).as_u64();
	let ops_per_second = if actual_duration > 0 {
		(operations * 1000) / actual_duration
	} else {
		0
	};

	Ok(StressTestResult {
		test_type: StressTestType::FileSystem,
		duration_jiffies: actual_duration,
		operations_completed: operations,
		operations_per_second: ops_per_second,
		errors_encountered: errors,
		details: format!("Created and deleted {} files", file_counter),
	})
}

/// Combined stress test
pub fn combined_stress_test(duration_seconds: u64) -> Result<Vec<StressTestResult>> {
	let mut results = Vec::new();

	// Run tests in sequence (parallel would be more stressful but harder to
	// implement)
	let per_test_duration = duration_seconds / 3;

	if let Ok(result) = memory_stress_test(per_test_duration) {
		results.push(result);
	}

	if let Ok(result) = cpu_stress_test(per_test_duration) {
		results.push(result);
	}

	if let Ok(result) = filesystem_stress_test(per_test_duration) {
		results.push(result);
	}

	Ok(results)
}

/// Generate system load for testing purposes
pub fn generate_load(test_type: StressTestType, duration_seconds: u64) -> Result<StressTestResult> {
	// Add diagnostic entry about starting stress test
	crate::diagnostics::add_diagnostic(
		crate::diagnostics::DiagnosticCategory::Kernel,
		crate::diagnostics::HealthStatus::Warning,
		&format!(
			"Starting {:?} stress test for {} seconds",
			test_type, duration_seconds
		),
		None,
	);

	let result = match test_type {
		StressTestType::Memory => memory_stress_test(duration_seconds),
		StressTestType::CPU => cpu_stress_test(duration_seconds),
		StressTestType::FileSystem => filesystem_stress_test(duration_seconds),
		StressTestType::IO | StressTestType::Network => {
			// Not implemented yet
			Err(crate::error::Error::NotSupported)
		}
		StressTestType::All => {
			// Run combined test and return the first result
			match combined_stress_test(duration_seconds) {
				Ok(results) if !results.is_empty() => Ok(results[0].clone()),
				Ok(_) => Err(crate::error::Error::Generic),
				Err(e) => Err(e),
			}
		}
	};

	// Add diagnostic entry about completing stress test
	match &result {
		Ok(test_result) => {
			crate::diagnostics::add_diagnostic(
				crate::diagnostics::DiagnosticCategory::Kernel,
				crate::diagnostics::HealthStatus::Healthy,
				&format!(
					"Completed {:?} stress test: {} ops/sec",
					test_result.test_type, test_result.operations_per_second
				),
				Some(&format!(
					"Duration: {}ms, Operations: {}, Errors: {}",
					test_result.duration_jiffies,
					test_result.operations_completed,
					test_result.errors_encountered
				)),
			);
		}
		Err(e) => {
			crate::diagnostics::add_diagnostic(
				crate::diagnostics::DiagnosticCategory::Kernel,
				crate::diagnostics::HealthStatus::Critical,
				&format!("Stress test failed: {}", e),
				None,
			);
		}
	}

	result
}

/// Format stress test results for display
pub fn format_stress_test_result(result: &StressTestResult) -> String {
	format!(
		"{:?} Stress Test Results:\n\
         Duration: {} ms\n\
         Operations: {}\n\
         Rate: {} ops/sec\n\
         Errors: {}\n\
         Details: {}",
		result.test_type,
		result.duration_jiffies,
		result.operations_completed,
		result.operations_per_second,
		result.errors_encountered,
		result.details
	)
}
