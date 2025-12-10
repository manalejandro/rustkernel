// SPDX-License-Identifier: GPL-2.0

//! Kernel memory allocation (kmalloc)

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::ptr::NonNull;

use crate::error::{Error, Result};
use crate::memory::allocator::{alloc_pages, free_pages, GfpFlags, PageFrameNumber};
use crate::sync::Spinlock;

/// Kmalloc size classes (powers of 2)
const KMALLOC_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
const MAX_KMALLOC_SIZE: usize = 4096;

/// Slab allocator for small kernel allocations
/// Uses physical addresses directly - they're identity-mapped in the first 1GB
struct SlabAllocator {
	size_classes: BTreeMap<usize, Vec<usize>>, // Store physical addresses
	allocated_blocks: BTreeMap<usize, usize>,  // Maps physical addresses to size classes
}

impl SlabAllocator {
	const fn new() -> Self {
		Self {
			size_classes: BTreeMap::new(),
			allocated_blocks: BTreeMap::new(),
		}
	}

	fn allocate(&mut self, size: usize) -> Result<*mut u8> {
		// Find appropriate size class
		let size_class = KMALLOC_SIZES
			.iter()
			.find(|&&s| s >= size)
			.copied()
			.unwrap_or(MAX_KMALLOC_SIZE);

		if size_class > MAX_KMALLOC_SIZE {
			return Err(Error::OutOfMemory);
		}

		// Try to get from free list
		if let Some(free_list) = self.size_classes.get_mut(&size_class) {
			if let Some(addr) = free_list.pop() {
				self.allocated_blocks.insert(addr, size_class);
				return Ok(addr as *mut u8);
			}
		}

		// Allocate new page and split it
		self.allocate_new_slab(size_class)
	}

	fn allocate_new_slab(&mut self, size_class: usize) -> Result<*mut u8> {
		// Allocate a page using buddy allocator
		let pfn = alloc_pages(0, GfpFlags::KERNEL)?;
		let page_addr = pfn.to_phys_addr().as_usize();

		// Split page into blocks of size_class
		let blocks_per_page = 4096 / size_class;
		let free_list = self.size_classes.entry(size_class).or_insert_with(Vec::new);

		for i in 1..blocks_per_page {
			let block_addr = page_addr + (i * size_class);
			free_list.push(block_addr);
		}

		// Return the first block
		self.allocated_blocks.insert(page_addr, size_class);
		Ok(page_addr as *mut u8)
	}

	fn deallocate(&mut self, ptr: *mut u8) -> Result<()> {
		let addr = ptr as usize;
		if let Some(size_class) = self.allocated_blocks.remove(&addr) {
			let free_list =
				self.size_classes.entry(size_class).or_insert_with(Vec::new);
			free_list.push(addr);
			Ok(())
		} else {
			Err(Error::InvalidArgument)
		}
	}

	pub fn stats(&self) -> (usize, usize, usize) {
		let mut allocated_count = 0;
		let mut allocated_bytes = 0;
		for (_, size_class) in &self.allocated_blocks {
			allocated_count += 1;
			allocated_bytes += size_class;
		}

		let mut free_count = 0;
		for (_, free_list) in &self.size_classes {
			free_count += free_list.len();
		}

		(allocated_count, allocated_bytes, free_count)
	}
}

static SLAB_ALLOCATOR: Spinlock<SlabAllocator> = Spinlock::new(SlabAllocator::new());

/// Get kmalloc statistics
pub fn get_stats() -> (usize, usize, usize) {
	let allocator = SLAB_ALLOCATOR.lock();
	allocator.stats()
}

/// Allocate kernel memory
pub fn kmalloc(size: usize) -> Result<*mut u8> {
	// Increment performance counter
	crate::perf::counters::inc_memory_allocs();

	if size == 0 {
		return Err(Error::InvalidArgument);
	}

	if size <= MAX_KMALLOC_SIZE {
		// Use slab allocator for small allocations
		let mut allocator = SLAB_ALLOCATOR.lock();
		allocator.allocate(size)
	} else {
		// Use buddy allocator for large allocations
		let pages_needed = (size + 4095) / 4096;
		let order = pages_needed.next_power_of_two().trailing_zeros() as usize;
		let pfn = alloc_pages(order, GfpFlags::KERNEL)?;
		Ok(pfn.to_phys_addr().as_usize() as *mut u8)
	}
}

/// Free kernel memory
pub fn kfree(ptr: *mut u8) {
	if ptr.is_null() {
		return;
	}

	// Try slab allocator first
	if let Ok(()) = SLAB_ALLOCATOR.lock().deallocate(ptr) {
		return;
	}

	// Try buddy allocator for large allocations
	// TODO: Keep track of large allocations to know how many pages to free
	// For now, assume single page
	if let Some(_page) = NonNull::new(ptr as *mut crate::memory::Page) {
		let pfn =
			PageFrameNumber::from_phys_addr(crate::types::PhysAddr::new(ptr as usize));
		free_pages(pfn, 0);
	}
}

/// Allocate zeroed kernel memory
pub fn kzalloc(size: usize) -> Result<*mut u8> {
	let ptr = kmalloc(size)?;
	unsafe {
		core::ptr::write_bytes(ptr, 0, size);
	}
	Ok(ptr)
}

/// Reallocate kernel memory
pub fn krealloc(ptr: *mut u8, old_size: usize, new_size: usize) -> Result<*mut u8> {
	if ptr.is_null() {
		return kmalloc(new_size);
	}

	if new_size == 0 {
		kfree(ptr);
		return Ok(core::ptr::null_mut());
	}

	let new_ptr = kmalloc(new_size)?;
	let copy_size = core::cmp::min(old_size, new_size);

	unsafe {
		core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
	}

	kfree(ptr);
	Ok(new_ptr)
}

/// Initialize the slab allocator
pub fn init() -> Result<()> {
	// No initialization needed - we use physical addresses directly
	Ok(())
}
