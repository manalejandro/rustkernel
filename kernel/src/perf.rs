// SPDX-License-Identifier: GPL-2.0

//! Performance monitoring and profiling

use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

use crate::error::Result;
use crate::sync::Spinlock;
use crate::types::Jiffies;

/// Performance counter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CounterType {
	CpuCycles,
	Instructions,
	CacheMisses,
	BranchMisses,
	PageFaults,
	ContextSwitches,
	Interrupts,
	SystemCalls,
	MemoryAllocations,
	FileOperations,
	NetworkPackets,
	Custom(u32),
}

/// Performance event
#[derive(Debug, Clone)]
pub struct PerfEvent {
	pub counter_type: CounterType,
	pub count: u64,
	pub timestamp: Jiffies,
	pub pid: Option<u32>,
	pub cpu: Option<u32>,
}

/// Performance counter
#[derive(Debug)]
pub struct PerfCounter {
	pub counter_type: CounterType,
	pub value: AtomicU64,
	pub enabled: bool,
	pub description: String,
}

impl PerfCounter {
	pub fn new(counter_type: CounterType, description: String) -> Self {
		Self {
			counter_type,
			value: AtomicU64::new(0),
			enabled: true,
			description,
		}
	}

	pub fn increment(&self) {
		if self.enabled {
			self.value.fetch_add(1, Ordering::Relaxed);
		}
	}

	pub fn add(&self, value: u64) {
		if self.enabled {
			self.value.fetch_add(value, Ordering::Relaxed);
		}
	}

	pub fn get(&self) -> u64 {
		self.value.load(Ordering::Relaxed)
	}

	pub fn reset(&self) {
		self.value.store(0, Ordering::Relaxed);
	}

	pub fn enable(&mut self) {
		self.enabled = true;
	}

	pub fn disable(&mut self) {
		self.enabled = false;
	}
}

/// Performance monitoring subsystem
pub struct PerfMonitor {
	counters: BTreeMap<CounterType, PerfCounter>,
	events: Vec<PerfEvent>,
	max_events: usize,
}

impl PerfMonitor {
	pub const fn new() -> Self {
		Self {
			counters: BTreeMap::new(),
			events: Vec::new(),
			max_events: 10000,
		}
	}

	pub fn init(&mut self) {
		// Initialize standard counters
		self.add_counter(CounterType::CpuCycles, "CPU cycles executed".into());
		self.add_counter(CounterType::Instructions, "Instructions executed".into());
		self.add_counter(CounterType::CacheMisses, "Cache misses".into());
		self.add_counter(CounterType::BranchMisses, "Branch prediction misses".into());
		self.add_counter(CounterType::PageFaults, "Page faults".into());
		self.add_counter(CounterType::ContextSwitches, "Context switches".into());
		self.add_counter(CounterType::Interrupts, "Interrupts handled".into());
		self.add_counter(CounterType::SystemCalls, "System calls".into());
		self.add_counter(CounterType::MemoryAllocations, "Memory allocations".into());
		self.add_counter(CounterType::FileOperations, "File operations".into());
		self.add_counter(CounterType::NetworkPackets, "Network packets".into());
	}

	pub fn add_counter(&mut self, counter_type: CounterType, description: String) {
		let counter = PerfCounter::new(counter_type, description);
		self.counters.insert(counter_type, counter);
	}

	pub fn increment_counter(&self, counter_type: CounterType) {
		if let Some(counter) = self.counters.get(&counter_type) {
			counter.increment();
		}
	}

	pub fn add_to_counter(&self, counter_type: CounterType, value: u64) {
		if let Some(counter) = self.counters.get(&counter_type) {
			counter.add(value);
		}
	}

	pub fn get_counter(&self, counter_type: CounterType) -> Option<u64> {
		self.counters.get(&counter_type).map(|c| c.get())
	}

	pub fn reset_counter(&self, counter_type: CounterType) {
		if let Some(counter) = self.counters.get(&counter_type) {
			counter.reset();
		}
	}

	pub fn record_event(&mut self, event: PerfEvent) {
		if self.events.len() >= self.max_events {
			self.events.remove(0); // Remove oldest event
		}
		self.events.push(event);
	}

	pub fn get_events(&self) -> &[PerfEvent] {
		&self.events
	}

	pub fn clear_events(&mut self) {
		self.events.clear();
	}

	pub fn get_counters(&self) -> &BTreeMap<CounterType, PerfCounter> {
		&self.counters
	}

	pub fn generate_report(&self) -> String {
		let mut report = String::from("Performance Monitor Report\n");
		report.push_str("==========================\n\n");

		for (counter_type, counter) in &self.counters {
			report.push_str(&format!(
				"{:?}: {} ({})\n",
				counter_type,
				counter.get(),
				counter.description
			));
		}

		report.push_str(&format!("\nTotal events recorded: {}\n", self.events.len()));

		if !self.events.is_empty() {
			report.push_str("\nRecent events:\n");
			for event in self.events.iter().rev().take(10) {
				report.push_str(&format!(
					"  {:?}: {} at {:?}\n",
					event.counter_type, event.count, event.timestamp
				));
			}
		}

		report
	}
}

/// Global performance monitor
static PERF_MONITOR: Spinlock<Option<PerfMonitor>> = Spinlock::new(None);

/// Initialize performance monitoring
pub fn init_perf_monitor() -> Result<()> {
	let mut monitor = PERF_MONITOR.lock();
	*monitor = Some(PerfMonitor::new());
	if let Some(ref mut m) = *monitor {
		m.init();
	}
	crate::info!("Performance monitoring initialized");
	Ok(())
}

/// Increment a performance counter
pub fn perf_counter_inc(counter_type: CounterType) {
	let monitor = PERF_MONITOR.lock();
	if let Some(ref m) = *monitor {
		m.increment_counter(counter_type);
	}
}

/// Add to a performance counter
pub fn perf_counter_add(counter_type: CounterType, value: u64) {
	let monitor = PERF_MONITOR.lock();
	if let Some(ref m) = *monitor {
		m.add_to_counter(counter_type, value);
	}
}

/// Get performance counter value
pub fn perf_counter_get(counter_type: CounterType) -> Option<u64> {
	let monitor = PERF_MONITOR.lock();
	if let Some(ref m) = *monitor {
		m.get_counter(counter_type)
	} else {
		None
	}
}

/// Reset performance counter
pub fn perf_counter_reset(counter_type: CounterType) {
	let monitor = PERF_MONITOR.lock();
	if let Some(ref m) = *monitor {
		m.reset_counter(counter_type);
	}
}

/// Record performance event
pub fn perf_event_record(counter_type: CounterType, count: u64) {
	let mut monitor = PERF_MONITOR.lock();
	if let Some(ref mut m) = *monitor {
		let event = PerfEvent {
			counter_type,
			count,
			timestamp: crate::time::get_jiffies(),
			pid: crate::process::current_process_pid().map(|p| p.0),
			cpu: Some(0), // TODO: Get current CPU ID
		};
		m.record_event(event);
	}
}

/// Generate performance report
pub fn perf_generate_report() -> String {
	let monitor = PERF_MONITOR.lock();
	if let Some(ref m) = *monitor {
		m.generate_report()
	} else {
		"Performance monitoring not initialized".into()
	}
}

/// Clear performance events
pub fn perf_clear_events() {
	let mut monitor = PERF_MONITOR.lock();
	if let Some(ref mut m) = *monitor {
		m.clear_events();
	}
}

/// Performance measurement macro
#[macro_export]
macro_rules! perf_measure {
	($counter_type:expr, $code:block) => {{
		let start = crate::time::get_jiffies();
		let result = $code;
		let end = crate::time::get_jiffies();
		crate::perf::perf_counter_add($counter_type, (end - start).as_u64());
		result
	}};
}

/// Convenience functions for common performance counters
pub mod counters {
	use super::*;

	pub fn inc_page_faults() {
		perf_counter_inc(CounterType::PageFaults);
	}

	pub fn inc_context_switches() {
		perf_counter_inc(CounterType::ContextSwitches);
	}

	pub fn inc_interrupts() {
		perf_counter_inc(CounterType::Interrupts);
	}

	pub fn inc_syscalls() {
		perf_counter_inc(CounterType::SystemCalls);
	}

	pub fn inc_memory_allocs() {
		perf_counter_inc(CounterType::MemoryAllocations);
	}

	pub fn inc_file_ops() {
		perf_counter_inc(CounterType::FileOperations);
	}

	pub fn inc_network_packets() {
		perf_counter_inc(CounterType::NetworkPackets);
	}
}
