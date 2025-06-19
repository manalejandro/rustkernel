// SPDX-License-Identifier: GPL-2.0

//! Page frame allocator

use alloc::collections::BTreeSet;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::error::{Error, Result};
use crate::sync::Spinlock;
use crate::types::{Pfn, PhysAddr};

/// Page structure - similar to Linux struct page
#[derive(Debug)]
pub struct Page {
	/// Page frame number
	pub pfn: Pfn,
	/// Page flags
	pub flags: AtomicU32,
	/// Reference count
	pub count: AtomicU32,
	/// Virtual address if mapped
	pub virtual_addr: Option<crate::types::VirtAddr>,
}

impl Page {
	/// Create a new page
	pub fn new(pfn: Pfn) -> Self {
		Self {
			pfn,
			flags: AtomicU32::new(0),
			count: AtomicU32::new(1),
			virtual_addr: None,
		}
	}

	/// Get physical address
	pub fn phys_addr(&self) -> PhysAddr {
		PhysAddr(self.pfn.0 * 4096) // Assuming 4KB pages
	}

	/// Get page flags
	pub fn flags(&self) -> u32 {
		self.flags.load(Ordering::Relaxed)
	}

	/// Set page flags
	pub fn set_flags(&self, flags: u32) {
		self.flags.store(flags, Ordering::Relaxed);
	}

	/// Get reference count
	pub fn count(&self) -> u32 {
		self.count.load(Ordering::Relaxed)
	}

	/// Increment reference count
	pub fn get(&self) -> u32 {
		self.count.fetch_add(1, Ordering::Relaxed) + 1
	}

	/// Decrement reference count
	pub fn put(&self) -> u32 {
		let old_count = self.count.fetch_sub(1, Ordering::Relaxed);
		if old_count == 1 {
			// Last reference - page can be freed
			// TODO: Add to free list
		}
		old_count - 1
	}
}

/// Page flags (Linux compatible)
pub mod page_flags {
	pub const PG_LOCKED: u32 = 0;
	pub const PG_ERROR: u32 = 1;
	pub const PG_REFERENCED: u32 = 2;
	pub const PG_UPTODATE: u32 = 3;
	pub const PG_DIRTY: u32 = 4;
	pub const PG_LRU: u32 = 5;
	pub const PG_ACTIVE: u32 = 6;
	pub const PG_SLAB: u32 = 7;
	pub const PG_OWNER_PRIV_1: u32 = 8;
	pub const PG_ARCH_1: u32 = 9;
	pub const PG_RESERVED: u32 = 10;
	pub const PG_PRIVATE: u32 = 11;
	pub const PG_PRIVATE_2: u32 = 12;
	pub const PG_WRITEBACK: u32 = 13;
	pub const PG_HEAD: u32 = 14;
	pub const PG_SWAPCACHE: u32 = 15;
	pub const PG_MAPPEDTODISK: u32 = 16;
	pub const PG_RECLAIM: u32 = 17;
	pub const PG_SWAPBACKED: u32 = 18;
	pub const PG_UNEVICTABLE: u32 = 19;
}

/// Page frame allocator
pub static PAGE_ALLOCATOR: Spinlock<PageAllocator> = Spinlock::new(PageAllocator::new());

/// Page allocator implementation
pub struct PageAllocator {
	free_pages: BTreeSet<Pfn>,
	total_pages: usize,
	allocated_pages: usize,
}

impl PageAllocator {
	pub const fn new() -> Self {
		Self {
			free_pages: BTreeSet::new(),
			total_pages: 0,
			allocated_pages: 0,
		}
	}

	/// Add a range of pages to the free list
	pub fn add_free_range(&mut self, start: Pfn, count: usize) {
		for i in 0..count {
			self.free_pages.insert(Pfn(start.0 + i));
		}
		self.total_pages += count;
	}

	/// Allocate a single page
	fn alloc_page(&mut self) -> Result<Pfn> {
		if let Some(pfn) = self.free_pages.iter().next().copied() {
			self.free_pages.remove(&pfn);
			self.allocated_pages += 1;
			Ok(pfn)
		} else {
			Err(Error::OutOfMemory)
		}
	}

	/// Free a single page
	fn free_page(&mut self, pfn: Pfn) {
		if self.free_pages.insert(pfn) {
			self.allocated_pages -= 1;
		}
	}

	/// Get statistics
	fn stats(&self) -> (usize, usize, usize) {
		(
			self.total_pages,
			self.allocated_pages,
			self.free_pages.len(),
		)
	}
}

/// Initialize the page allocator
pub fn init() -> Result<()> {
	let mut allocator = PAGE_ALLOCATOR.lock();

	// TODO: Get memory map from bootloader/firmware
	// For now, add a dummy range
	let start_pfn = Pfn(0x1000); // Start at 16MB
	let count = 0x10000; // 256MB worth of pages

	allocator.add_free_range(start_pfn, count);

	Ok(())
}

/// Add a range of free pages by physical address
pub fn add_free_range(start_addr: PhysAddr, end_addr: PhysAddr) -> Result<()> {
	let start_pfn = Pfn::from_phys_addr(start_addr);
	let end_pfn = Pfn::from_phys_addr(end_addr);

	if end_pfn.0 <= start_pfn.0 {
		return Err(crate::error::Error::InvalidArgument);
	}

	let count = end_pfn.0 - start_pfn.0;
	let mut allocator = PAGE_ALLOCATOR.lock();
	allocator.add_free_range(start_pfn, count);

	Ok(())
}

/// Allocate a page of physical memory
pub fn alloc_page() -> Result<PhysAddr> {
	let mut allocator = PAGE_ALLOCATOR.lock();
	let pfn = allocator.alloc_page()?;
	Ok(pfn.to_phys_addr())
}

/// Allocate a page of physical memory (alias for alloc_page)
pub fn allocate_page() -> Result<PhysAddr> {
	alloc_page()
}

/// Free a page of physical memory
pub fn free_page(addr: PhysAddr) {
	let pfn = Pfn::from_phys_addr(addr);
	let mut allocator = PAGE_ALLOCATOR.lock();
	allocator.free_page(pfn);
}

/// Get page allocator statistics
pub fn stats() -> (usize, usize, usize) {
	let allocator = PAGE_ALLOCATOR.lock();
	allocator.stats()
}
