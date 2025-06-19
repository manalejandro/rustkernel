// SPDX-License-Identifier: GPL-2.0

//! Memory allocator implementation - Enhanced with buddy allocator

use alloc::vec::Vec;

use crate::error::{Error, Result};
use crate::memory::advanced_allocator::ALLOCATOR;
use crate::sync::Spinlock;
use crate::types::{PhysAddr, VirtAddr, PAGE_SIZE};

/// Maximum order for buddy allocator (2^MAX_ORDER pages)
const MAX_ORDER: usize = 11;

/// Page frame number
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageFrameNumber(pub usize);

impl PageFrameNumber {
	pub fn to_phys_addr(self) -> PhysAddr {
		PhysAddr::new(self.0 * PAGE_SIZE)
	}

	pub fn from_phys_addr(addr: PhysAddr) -> Self {
		Self(addr.as_usize() / PAGE_SIZE)
	}
}

/// Page allocation flags
#[derive(Debug, Clone, Copy)]
pub struct GfpFlags(pub u32);

impl GfpFlags {
	pub const KERNEL: Self = Self(0x01);
	pub const USER: Self = Self(0x02);
	pub const ATOMIC: Self = Self(0x04);
	pub const ZERO: Self = Self(0x08);
	pub const DMA: Self = Self(0x10);
	pub const HIGHMEM: Self = Self(0x20);
}

/// Free page block in buddy allocator
#[derive(Debug)]
struct FreeBlock {
	pfn: PageFrameNumber,
	#[allow(dead_code)]
	order: usize,
}

/// Simple buddy allocator for page allocation
pub struct BuddyAllocator {
	/// Free lists for each order
	free_lists: [Vec<FreeBlock>; MAX_ORDER],
	/// Total number of pages
	total_pages: usize,
	/// Base physical address
	#[allow(dead_code)]
	base_addr: PhysAddr,
}

impl BuddyAllocator {
	pub fn new(base_addr: PhysAddr, total_pages: usize) -> Self {
		const EMPTY_VEC: Vec<FreeBlock> = Vec::new();

		Self {
			free_lists: [EMPTY_VEC; MAX_ORDER],
			total_pages,
			base_addr,
		}
	}

	/// Add a free memory region to the allocator
	pub fn add_free_region(&mut self, start_pfn: PageFrameNumber, num_pages: usize) {
		// Simple implementation: add as single-page blocks
		for i in 0..num_pages {
			self.free_lists[0].push(FreeBlock {
				pfn: PageFrameNumber(start_pfn.0 + i),
				order: 0,
			});
		}
	}

	/// Allocate pages of given order
	pub fn alloc_pages(&mut self, order: usize) -> Result<PageFrameNumber> {
		if order >= MAX_ORDER {
			return Err(Error::InvalidArgument);
		}

		// Try to find a free block of the requested order
		if let Some(block) = self.free_lists[order].pop() {
			return Ok(block.pfn);
		}

		// For simplicity, just allocate from order 0 if we need higher orders
		if order > 0 {
			// Try to get multiple single pages (simplified approach)
			let pages_needed = 1 << order;
			if self.free_lists[0].len() >= pages_needed {
				let first_pfn = self.free_lists[0].pop().unwrap().pfn;
				// Remove additional pages
				for _ in 1..pages_needed {
					if self.free_lists[0].is_empty() {
						// Put back the first page if we can't get enough
						self.free_lists[0].push(FreeBlock {
							pfn: first_pfn,
							order: 0,
						});
						return Err(Error::OutOfMemory);
					}
					self.free_lists[0].pop();
				}
				return Ok(first_pfn);
			}
		}

		Err(Error::OutOfMemory)
	}

	/// Free pages
	pub fn free_pages(&mut self, pfn: PageFrameNumber, order: usize) {
		if order >= MAX_ORDER {
			return;
		}

		// Simple implementation: add back to the appropriate order
		let pages_to_free = 1 << order;
		for i in 0..pages_to_free {
			self.free_lists[0].push(FreeBlock {
				pfn: PageFrameNumber(pfn.0 + i),
				order: 0,
			});
		}
	}

	/// Get free page count
	pub fn free_pages_count(&self) -> usize {
		let mut count = 0;
		for order in 0..MAX_ORDER {
			count += self.free_lists[order].len() * (1 << order);
		}
		count
	}
}

/// Global buddy allocator for page allocation
static PAGE_ALLOCATOR: Spinlock<Option<BuddyAllocator>> = Spinlock::new(None);

/// Heap start address (will be set during initialization)
static mut HEAP_START: usize = 0;
static mut HEAP_SIZE: usize = 0;

/// Initialize the allocators
pub fn init() -> Result<()> {
	// Initialize heap allocator
	let heap_start = 0x_4444_4444_0000;
	let heap_size = 100 * 1024; // 100 KB

	unsafe {
		HEAP_START = heap_start;
		HEAP_SIZE = heap_size;
		crate::memory::advanced_allocator::init_advanced_allocator(heap_start, heap_size);
	}

	// Initialize page allocator
	// For demo purposes, assume we have 1024 pages starting at 1MB
	let page_base = PhysAddr::new(0x100000); // 1MB
	let total_pages = 1024;

	let mut buddy = BuddyAllocator::new(page_base, total_pages);
	buddy.add_free_region(
		PageFrameNumber(page_base.as_usize() / PAGE_SIZE),
		total_pages,
	);

	*PAGE_ALLOCATOR.lock() = Some(buddy);

	Ok(())
}

/// Get heap statistics
pub fn heap_stats() -> (usize, usize) {
	unsafe { (HEAP_START, HEAP_SIZE) }
}

/// Linux-compatible page allocation functions
pub fn alloc_pages(order: usize, flags: GfpFlags) -> Result<PageFrameNumber> {
	let mut allocator = PAGE_ALLOCATOR.lock();
	if let Some(ref mut alloc) = *allocator {
		alloc.alloc_pages(order)
	} else {
		Err(Error::NotInitialized)
	}
}

pub fn free_pages(pfn: PageFrameNumber, order: usize) {
	let mut allocator = PAGE_ALLOCATOR.lock();
	if let Some(ref mut alloc) = *allocator {
		alloc.free_pages(pfn, order);
	}
}

/// Allocate a single page
pub fn get_free_page(flags: GfpFlags) -> Result<VirtAddr> {
	let pfn = alloc_pages(0, flags)?;
	Ok(VirtAddr::new(pfn.to_phys_addr().as_usize()))
}

/// Free a single page
pub fn free_page(addr: VirtAddr) {
	let pfn = PageFrameNumber::from_phys_addr(PhysAddr::new(addr.as_usize()));
	free_pages(pfn, 0);
}

/// Get page allocator statistics
pub fn page_alloc_stats() -> Option<(usize, usize)> {
	let allocator = PAGE_ALLOCATOR.lock();
	if let Some(ref alloc) = *allocator {
		Some((alloc.total_pages, alloc.free_pages_count()))
	} else {
		None
	}
}
