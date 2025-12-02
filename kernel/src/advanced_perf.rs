// SPDX-License-Identifier: GPL-2.0

//! Advanced performance monitoring and profiling system

use alloc::{
	collections::BTreeMap,
	string::{String, ToString},
	vec::Vec,
};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::error::{Error, Result};
use crate::sync::Spinlock;
use crate::types::Jiffies;

/// Performance counter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CounterType {
	CpuCycles,
	Instructions,
	CacheMisses,
	PageFaults,
	ContextSwitches,
	SystemCalls,
	Interrupts,
	MemoryAllocations,
	DiskReads,
	DiskWrites,
	NetworkPackets,
	Custom(u32),
}

/// Performance event structure
#[derive(Debug, Clone)]
pub struct PerformanceEvent {
	pub counter_type: CounterType,
	pub value: u64,
	pub timestamp: Jiffies,
	pub process_id: Option<u32>,
	pub thread_id: Option<u32>,
	pub cpu_id: Option<u8>,
}

/// Performance counter
#[derive(Debug)]
pub struct PerformanceCounter {
	pub counter_type: CounterType,
	pub value: AtomicU64,
	pub enabled: AtomicBool,
	pub last_reset: AtomicU64,
	pub description: String,
}

impl PerformanceCounter {
	pub fn new(counter_type: CounterType, description: String) -> Self {
		Self {
			counter_type,
			value: AtomicU64::new(0),
			enabled: AtomicBool::new(true),
			last_reset: AtomicU64::new(crate::time::get_jiffies().0),
			description,
		}
	}

	pub fn increment(&self, amount: u64) {
		if self.enabled.load(Ordering::Relaxed) {
			self.value.fetch_add(amount, Ordering::Relaxed);
		}
	}

	pub fn get_value(&self) -> u64 {
		self.value.load(Ordering::Relaxed)
	}

	pub fn reset(&self) {
		self.value.store(0, Ordering::Relaxed);
		self.last_reset
			.store(crate::time::get_jiffies().0, Ordering::Relaxed);
	}

	pub fn enable(&self) {
		self.enabled.store(true, Ordering::Relaxed);
	}

	pub fn disable(&self) {
		self.enabled.store(false, Ordering::Relaxed);
	}
}

/// Performance profiler for function/code block profiling
#[derive(Debug)]
pub struct Profiler {
	pub name: String,
	pub call_count: AtomicU64,
	pub total_time: AtomicU64,
	pub min_time: AtomicU64,
	pub max_time: AtomicU64,
	pub enabled: AtomicBool,
}

impl Profiler {
	pub fn new(name: String) -> Self {
		Self {
			name,
			call_count: AtomicU64::new(0),
			total_time: AtomicU64::new(0),
			min_time: AtomicU64::new(u64::MAX),
			max_time: AtomicU64::new(0),
			enabled: AtomicBool::new(true),
		}
	}

	pub fn record_execution(&self, duration: u64) {
		if !self.enabled.load(Ordering::Relaxed) {
			return;
		}

		self.call_count.fetch_add(1, Ordering::Relaxed);
		self.total_time.fetch_add(duration, Ordering::Relaxed);

		// Update min time
		let mut current_min = self.min_time.load(Ordering::Relaxed);
		while duration < current_min {
			match self.min_time.compare_exchange_weak(
				current_min,
				duration,
				Ordering::Relaxed,
				Ordering::Relaxed,
			) {
				Ok(_) => break,
				Err(x) => current_min = x,
			}
		}

		// Update max time
		let mut current_max = self.max_time.load(Ordering::Relaxed);
		while duration > current_max {
			match self.max_time.compare_exchange_weak(
				current_max,
				duration,
				Ordering::Relaxed,
				Ordering::Relaxed,
			) {
				Ok(_) => break,
				Err(x) => current_max = x,
			}
		}
	}

	pub fn get_stats(&self) -> ProfilerStats {
		let call_count = self.call_count.load(Ordering::Relaxed);
		let total_time = self.total_time.load(Ordering::Relaxed);

		ProfilerStats {
			name: self.name.clone(),
			call_count,
			total_time,
			average_time: if call_count > 0 {
				total_time / call_count
			} else {
				0
			},
			min_time: if call_count > 0 {
				self.min_time.load(Ordering::Relaxed)
			} else {
				0
			},
			max_time: self.max_time.load(Ordering::Relaxed),
		}
	}

	pub fn reset(&self) {
		self.call_count.store(0, Ordering::Relaxed);
		self.total_time.store(0, Ordering::Relaxed);
		self.min_time.store(u64::MAX, Ordering::Relaxed);
		self.max_time.store(0, Ordering::Relaxed);
	}
}

/// Profiler statistics snapshot
#[derive(Debug, Clone)]
pub struct ProfilerStats {
	pub name: String,
	pub call_count: u64,
	pub total_time: u64,
	pub average_time: u64,
	pub min_time: u64,
	pub max_time: u64,
}

/// System-wide performance monitoring
pub struct PerformanceMonitor {
	counters: Spinlock<BTreeMap<CounterType, PerformanceCounter>>,
	profilers: Spinlock<BTreeMap<String, Profiler>>,
	events: Spinlock<Vec<PerformanceEvent>>,
	max_events: usize,
	monitoring_enabled: AtomicBool,
}

impl PerformanceMonitor {
	pub const fn new() -> Self {
		Self {
			counters: Spinlock::new(BTreeMap::new()),
			profilers: Spinlock::new(BTreeMap::new()),
			events: Spinlock::new(Vec::new()),
			max_events: 10000,
			monitoring_enabled: AtomicBool::new(true),
		}
	}

	/// Initialize default performance counters
	pub fn init(&self) -> Result<()> {
		let mut counters = self.counters.lock();

		counters.insert(
			CounterType::ContextSwitches,
			PerformanceCounter::new(
				CounterType::ContextSwitches,
				"Context switches".to_string(),
			),
		);
		counters.insert(
			CounterType::SystemCalls,
			PerformanceCounter::new(
				CounterType::SystemCalls,
				"System calls".to_string(),
			),
		);
		counters.insert(
			CounterType::Interrupts,
			PerformanceCounter::new(
				CounterType::Interrupts,
				"Hardware interrupts".to_string(),
			),
		);
		counters.insert(
			CounterType::MemoryAllocations,
			PerformanceCounter::new(
				CounterType::MemoryAllocations,
				"Memory allocations".to_string(),
			),
		);
		counters.insert(
			CounterType::PageFaults,
			PerformanceCounter::new(CounterType::PageFaults, "Page faults".to_string()),
		);

		drop(counters);
		crate::info!("Performance monitoring initialized");
		Ok(())
	}

	/// Record performance event
	pub fn record_event(&self, counter_type: CounterType, value: u64) {
		if !self.monitoring_enabled.load(Ordering::Relaxed) {
			return;
		}

		// Update counter
		if let Some(counter) = self.counters.lock().get(&counter_type) {
			counter.increment(value);
		}

		// Record event
		let event = PerformanceEvent {
			counter_type,
			value,
			timestamp: crate::time::get_jiffies(),
			process_id: None, // TODO: Get current process ID
			thread_id: None,  // TODO: Get current thread ID
			cpu_id: None,     // TODO: Get current CPU ID
		};

		let mut events = self.events.lock();
		if events.len() >= self.max_events {
			events.remove(0); // Remove oldest event
		}
		events.push(event);
	}

	/// Get performance counter value
	pub fn get_counter(&self, counter_type: CounterType) -> Option<u64> {
		self.counters
			.lock()
			.get(&counter_type)
			.map(|c| c.get_value())
	}

	/// Reset performance counter
	pub fn reset_counter(&self, counter_type: CounterType) -> Result<()> {
		match self.counters.lock().get(&counter_type) {
			Some(counter) => {
				counter.reset();
				Ok(())
			}
			None => Err(Error::NotFound),
		}
	}

	/// Create or get profiler
	pub fn get_profiler(&self, name: String) -> Result<()> {
		let mut profilers = self.profilers.lock();
		if !profilers.contains_key(&name) {
			profilers.insert(name.clone(), Profiler::new(name));
		}
		Ok(())
	}

	/// Record profiler execution
	pub fn record_profiler(&self, name: &str, duration: u64) -> Result<()> {
		match self.profilers.lock().get(name) {
			Some(profiler) => {
				profiler.record_execution(duration);
				Ok(())
			}
			None => Err(Error::NotFound),
		}
	}

	/// Get all profiler statistics
	pub fn get_profiler_stats(&self) -> Vec<ProfilerStats> {
		self.profilers
			.lock()
			.values()
			.map(|p| p.get_stats())
			.collect()
	}

	/// Get performance summary
	pub fn get_summary(&self) -> PerformanceSummary {
		let counters = self.counters.lock();
		let counter_values: Vec<_> =
			counters.iter().map(|(t, c)| (*t, c.get_value())).collect();
		drop(counters);

		let profiler_stats = self.get_profiler_stats();
		let event_count = self.events.lock().len();

		PerformanceSummary {
			counters: counter_values,
			profilers: profiler_stats,
			total_events: event_count,
			monitoring_enabled: self.monitoring_enabled.load(Ordering::Relaxed),
		}
	}

	/// Enable/disable monitoring
	pub fn set_monitoring(&self, enabled: bool) {
		self.monitoring_enabled.store(enabled, Ordering::Relaxed);
	}

	/// Clear all events
	pub fn clear_events(&self) {
		self.events.lock().clear();
	}

	/// Reset all counters and profilers
	pub fn reset_all(&self) {
		for counter in self.counters.lock().values() {
			counter.reset();
		}

		for profiler in self.profilers.lock().values() {
			profiler.reset();
		}

		self.clear_events();
	}
}

/// Performance summary structure
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
	pub counters: Vec<(CounterType, u64)>,
	pub profilers: Vec<ProfilerStats>,
	pub total_events: usize,
	pub monitoring_enabled: bool,
}

/// RAII profiler guard for automatic timing
pub struct ProfileGuard {
	profiler_name: String,
	start_time: u64,
}

impl ProfileGuard {
	pub fn new(profiler_name: String) -> Result<Self> {
		// Ensure profiler exists
		PERFORMANCE_MONITOR.get_profiler(profiler_name.clone())?;

		Ok(Self {
			profiler_name,
			start_time: crate::time::get_jiffies().0,
		})
	}
}

impl Drop for ProfileGuard {
	fn drop(&mut self) {
		let end_time = crate::time::get_jiffies().0;
		let duration = end_time.saturating_sub(self.start_time);
		let _ = PERFORMANCE_MONITOR.record_profiler(&self.profiler_name, duration);
	}
}

/// Global performance monitor
static PERFORMANCE_MONITOR: PerformanceMonitor = PerformanceMonitor::new();

/// Initialize performance monitoring
pub fn init_performance_monitoring() -> Result<()> {
	PERFORMANCE_MONITOR.init()
}

/// Record performance event
pub fn record_event(counter_type: CounterType, value: u64) {
	PERFORMANCE_MONITOR.record_event(counter_type, value);
}

/// Get performance counter value
pub fn get_counter(counter_type: CounterType) -> Option<u64> {
	PERFORMANCE_MONITOR.get_counter(counter_type)
}

/// Reset performance counter
pub fn reset_counter(counter_type: CounterType) -> Result<()> {
	PERFORMANCE_MONITOR.reset_counter(counter_type)
}

/// Create profiler guard for automatic timing
pub fn profile(name: String) -> Result<ProfileGuard> {
	ProfileGuard::new(name)
}

/// Get performance summary
pub fn get_performance_summary() -> PerformanceSummary {
	PERFORMANCE_MONITOR.get_summary()
}

/// Enable/disable performance monitoring
pub fn set_monitoring_enabled(enabled: bool) {
	PERFORMANCE_MONITOR.set_monitoring(enabled);
}

/// Clear performance events
pub fn clear_performance_events() {
	PERFORMANCE_MONITOR.clear_events();
}

/// Reset all performance data
pub fn reset_all_performance_data() {
	PERFORMANCE_MONITOR.reset_all();
}

/// Profile function execution (returns RAII guard)
pub fn profile_function(function_name: &str) -> Result<ProfileGuard> {
	profile(function_name.to_string())
}

/// Convenience macros for performance monitoring
#[macro_export]
macro_rules! perf_counter {
	($counter_type:expr, $value:expr) => {
		$crate::advanced_perf::record_event($counter_type, $value);
	};
}

#[macro_export]
macro_rules! perf_profile {
	($name:expr, $code:block) => {{
		let _guard = $crate::advanced_perf::profile($name.to_string());
		$code
	}};
}
