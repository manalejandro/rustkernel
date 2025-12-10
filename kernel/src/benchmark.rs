// SPDX-License-Identifier: GPL-2.0

//! Kernel benchmark system

use alloc::{
	string::{String, ToString},
	vec,
	vec::Vec,
};

use crate::error::Result;
use crate::time::{get_jiffies, monotonic_time};
use crate::{info, warn};

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
	pub name: String,
	pub iterations: u64,
	pub total_time_ns: u64,
	pub avg_time_ns: u64,
	pub min_time_ns: u64,
	pub max_time_ns: u64,
}

impl BenchmarkResult {
	pub fn new(name: String, iterations: u64, times: &[u64]) -> Self {
		let total_time_ns = times.iter().sum();
		let avg_time_ns = total_time_ns / iterations;
		let min_time_ns = *times.iter().min().unwrap_or(&0);
		let max_time_ns = *times.iter().max().unwrap_or(&0);

		Self {
			name,
			iterations,
			total_time_ns,
			avg_time_ns,
			min_time_ns,
			max_time_ns,
		}
	}

	pub fn print(&self) {
		info!("Benchmark: {}", self.name);
		info!("  Iterations: {}", self.iterations);
		info!("  Total time: {} ns", self.total_time_ns);
		info!("  Average time: {} ns", self.avg_time_ns);
		info!("  Min time: {} ns", self.min_time_ns);
		info!("  Max time: {} ns", self.max_time_ns);
	}
}

/// Benchmark function type
pub type BenchmarkFn = fn();

/// Run a benchmark
pub fn benchmark(name: &str, iterations: u64, func: BenchmarkFn) -> BenchmarkResult {
	let mut times = Vec::new();

	info!("Running benchmark: {} ({} iterations)", name, iterations);

	for _i in 0..iterations {
		let start = monotonic_time();
		func();
		let end = monotonic_time();

		let elapsed_ns = (end.to_ns() as i64 - start.to_ns() as i64) as u64;
		times.push(elapsed_ns);
	}

	let result = BenchmarkResult::new(name.to_string(), iterations, &times);
	result.print();
	result
}

/// Memory allocation benchmark
fn bench_memory_alloc() {
	let _vec: Vec<u8> = Vec::with_capacity(1024);
}

/// Memory deallocation benchmark
fn bench_memory_dealloc() {
	let vec: Vec<u8> = Vec::with_capacity(1024);
	drop(vec);
}

/// Simple arithmetic benchmark
fn bench_arithmetic() {
	let mut result = 0u64;
	for i in 0..1000 {
		result = result.wrapping_add(i).wrapping_mul(2);
	}
	// Prevent optimization
	core::hint::black_box(result);
}

/// String operations benchmark
fn bench_string_ops() {
	let mut s = String::new();
	for i in 0..100 {
		s.push_str("test");
		s.push((b'0' + (i % 10) as u8) as char);
	}
	core::hint::black_box(s);
}

/// Interrupt enable/disable benchmark
fn bench_interrupt_toggle() {
	crate::interrupt::disable();
	crate::interrupt::enable();
}

/// Run all kernel benchmarks
pub fn run_all_benchmarks() -> Result<Vec<BenchmarkResult>> {
	info!("Running kernel performance benchmarks");

	let mut results = Vec::new();

	// Memory benchmarks
	results.push(benchmark("memory_alloc", 1000, bench_memory_alloc));
	results.push(benchmark("memory_dealloc", 1000, bench_memory_dealloc));

	// CPU benchmarks
	results.push(benchmark("arithmetic", 100, bench_arithmetic));
	results.push(benchmark("string_ops", 100, bench_string_ops));

	// System call benchmarks
	results.push(benchmark("interrupt_toggle", 1000, bench_interrupt_toggle));

	info!("Benchmark suite completed");
	Ok(results)
}

/// Run specific benchmark
pub fn run_benchmark(name: &str, iterations: u64) -> Result<BenchmarkResult> {
	match name {
		"memory_alloc" => Ok(benchmark(name, iterations, bench_memory_alloc)),
		"memory_dealloc" => Ok(benchmark(name, iterations, bench_memory_dealloc)),
		"arithmetic" => Ok(benchmark(name, iterations, bench_arithmetic)),
		"string_ops" => Ok(benchmark(name, iterations, bench_string_ops)),
		"interrupt_toggle" => Ok(benchmark(name, iterations, bench_interrupt_toggle)),
		_ => {
			warn!("Unknown benchmark: {}", name);
			Err(crate::error::Error::NotFound)
		}
	}
}

/// Get available benchmarks
pub fn list_benchmarks() -> Vec<&'static str> {
	vec![
		"memory_alloc",
		"memory_dealloc",
		"arithmetic",
		"string_ops",
		"interrupt_toggle",
	]
}

/// Performance stress test
pub fn stress_test(duration_seconds: u64) -> Result<()> {
	info!("Running stress test for {} seconds", duration_seconds);

	let start_jiffies = get_jiffies();
	let target_jiffies = start_jiffies.0 + (duration_seconds * crate::time::HZ);

	let mut iterations = 0u64;

	while get_jiffies().0 < target_jiffies {
		// Mix of different operations
		bench_arithmetic();
		bench_memory_alloc();
		bench_string_ops();
		bench_interrupt_toggle();

		iterations += 1;

		// Yield occasionally to prevent monopolizing CPU
		if iterations % 1000 == 0 {
			crate::kthread::kthread_yield();
		}
	}

	let elapsed_jiffies = get_jiffies().0 - start_jiffies.0;
	let ops_per_second = (iterations * crate::time::HZ) / elapsed_jiffies;

	info!("Stress test completed:");
	info!("  Duration: {} jiffies", elapsed_jiffies);
	info!("  Total iterations: {}", iterations);
	info!("  Operations per second: {}", ops_per_second);

	Ok(())
}
