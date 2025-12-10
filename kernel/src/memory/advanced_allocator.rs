// SPDX-License-Identifier: GPL-2.0

//! Advanced memory allocator with debugging and tracking capabilities

use alloc::{collections::BTreeMap, vec::Vec};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::error::{Error, Result};
use crate::sync::Spinlock;

/// Allocation tracking information
#[derive(Debug, Clone)]
pub struct AllocationInfo {
	pub size: usize,
	pub layout: Layout,
	pub timestamp: u64,
	pub caller: Option<usize>, // Return address for debugging
}

/// Memory allocation statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
	pub total_allocated: AtomicU64,
	pub total_freed: AtomicU64,
	pub current_allocated: AtomicU64,
	pub allocation_count: AtomicU64,
	pub free_count: AtomicU64,
	pub peak_usage: AtomicU64,
	pub fragmentation_events: AtomicU64,
}

/// Advanced allocator with tracking and debugging
pub struct AdvancedAllocator {
	base_allocator: linked_list_allocator::LockedHeap,
	allocations: Spinlock<BTreeMap<usize, AllocationInfo>>,
	stats: MemoryStats,
	debug_mode: AtomicU64, // Bitfield for debug features
}

impl AdvancedAllocator {
	/// Create new advanced allocator
	pub const fn new() -> Self {
		Self {
			base_allocator: linked_list_allocator::LockedHeap::empty(),
			allocations: Spinlock::new(BTreeMap::new()),
			stats: MemoryStats {
				total_allocated: AtomicU64::new(0),
				total_freed: AtomicU64::new(0),
				current_allocated: AtomicU64::new(0),
				allocation_count: AtomicU64::new(0),
				free_count: AtomicU64::new(0),
				peak_usage: AtomicU64::new(0),
				fragmentation_events: AtomicU64::new(0),
			},
			debug_mode: AtomicU64::new(0),
		}
	}

	/// Initialize the allocator with heap memory
	pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
		self.base_allocator
			.lock()
			.init(heap_start as *mut u8, heap_size);
	}

	/// Enable debug mode features
	pub fn set_debug_mode(&self, mode: u64) {
		self.debug_mode.store(mode, Ordering::Relaxed);
	}

	/// Get current memory statistics
	pub fn get_stats(&self) -> MemoryStatsSnapshot {
		MemoryStatsSnapshot {
			total_allocated: self.stats.total_allocated.load(Ordering::Relaxed),
			total_freed: self.stats.total_freed.load(Ordering::Relaxed),
			current_allocated: self.stats.current_allocated.load(Ordering::Relaxed),
			allocation_count: self.stats.allocation_count.load(Ordering::Relaxed),
			free_count: self.stats.free_count.load(Ordering::Relaxed),
			peak_usage: self.stats.peak_usage.load(Ordering::Relaxed),
			fragmentation_events: self
				.stats
				.fragmentation_events
				.load(Ordering::Relaxed),
			active_allocations: self.allocations.lock().len(),
		}
	}

	/// Get detailed allocation information
	pub fn get_allocations(&self) -> Vec<(usize, AllocationInfo)> {
		self.allocations
			.lock()
			.iter()
			.map(|(&addr, info)| (addr, info.clone()))
			.collect()
	}

	/// Check for memory leaks
	pub fn check_leaks(&self) -> Vec<(usize, AllocationInfo)> {
		let current_time = crate::time::get_jiffies();
		self.allocations
			.lock()
			.iter()
			.filter(|(_, info)| {
				current_time.0 > info.timestamp
					&& current_time.0 - info.timestamp > 10000
			}) // Old allocations
			.map(|(&addr, info)| (addr, info.clone()))
			.collect()
	}

	/// Defragment memory (placeholder for future implementation)
	pub fn defragment(&self) -> Result<usize> {
		// This would implement memory compaction
		// For now, just return that no bytes were moved
		Ok(0)
	}
}

unsafe impl GlobalAlloc for AdvancedAllocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let ptr = self.base_allocator.alloc(layout);

		if !ptr.is_null() {
			// Update statistics
			let size = layout.size() as u64;
			self.stats
				.total_allocated
				.fetch_add(size, Ordering::Relaxed);
			self.stats.allocation_count.fetch_add(1, Ordering::Relaxed);
			let current =
				self.stats
					.current_allocated
					.fetch_add(size, Ordering::Relaxed) + size;

			// Update peak usage
			let mut peak = self.stats.peak_usage.load(Ordering::Relaxed);
			while current > peak {
				match self.stats.peak_usage.compare_exchange_weak(
					peak,
					current,
					Ordering::Relaxed,
					Ordering::Relaxed,
				) {
					Ok(_) => break,
					Err(x) => peak = x,
				}
			}

			// Track allocation if debug mode is enabled
			if self.debug_mode.load(Ordering::Relaxed) & 1 != 0 {
				let info = AllocationInfo {
					size: layout.size(),
					layout,
					timestamp: crate::time::get_jiffies().0,
					caller: None, // TODO: Get return address
				};
				self.allocations.lock().insert(ptr as usize, info);
			}
		}

		ptr
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.base_allocator.dealloc(ptr, layout);

		if !ptr.is_null() {
			// Update statistics
			let size = layout.size() as u64;
			self.stats.total_freed.fetch_add(size, Ordering::Relaxed);
			self.stats.free_count.fetch_add(1, Ordering::Relaxed);
			self.stats
				.current_allocated
				.fetch_sub(size, Ordering::Relaxed);

			// Remove allocation tracking
			if self.debug_mode.load(Ordering::Relaxed) & 1 != 0 {
				self.allocations.lock().remove(&(ptr as usize));
			}
		}
	}
}

/// Snapshot of memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStatsSnapshot {
	pub total_allocated: u64,
	pub total_freed: u64,
	pub current_allocated: u64,
	pub allocation_count: u64,
	pub free_count: u64,
	pub peak_usage: u64,
	pub fragmentation_events: u64,
	pub active_allocations: usize,
}

/// Debug mode flags
pub mod debug_flags {
	pub const TRACK_ALLOCATIONS: u64 = 1 << 0;
	pub const DETECT_LEAKS: u64 = 1 << 1;
	pub const POISON_MEMORY: u64 = 1 << 2;
	pub const GUARD_PAGES: u64 = 1 << 3;
}

/// Global advanced allocator instance
#[global_allocator]
pub static ALLOCATOR: AdvancedAllocator = AdvancedAllocator::new();

/// Initialize the advanced allocator
pub unsafe fn init_advanced_allocator(heap_start: usize, heap_size: usize) {
	ALLOCATOR.init(heap_start, heap_size);
	ALLOCATOR.set_debug_mode(debug_flags::TRACK_ALLOCATIONS | debug_flags::DETECT_LEAKS);
	crate::info!("Advanced allocator initialized with {} bytes", heap_size);
}

/// Get global memory statistics
pub fn get_memory_stats() -> MemoryStatsSnapshot {
	ALLOCATOR.get_stats()
}

/// Get current allocations for debugging
pub fn get_current_allocations() -> Vec<(usize, AllocationInfo)> {
	ALLOCATOR.get_allocations()
}

/// Check for memory leaks
pub fn check_memory_leaks() -> Vec<(usize, AllocationInfo)> {
	ALLOCATOR.check_leaks()
}

/// Trigger memory defragmentation
pub fn defragment_memory() -> Result<usize> {
	ALLOCATOR.defragment()
}
