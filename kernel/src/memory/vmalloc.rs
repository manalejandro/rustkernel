// SPDX-License-Identifier: GPL-2.0

//! Virtual memory allocation

use alloc::collections::BTreeMap;
use core::ptr::NonNull;

use crate::error::{Error, Result};
use crate::memory::allocator::{alloc_pages, free_pages, GfpFlags, PageFrameNumber};
use crate::memory::page_table::{PageTableFlags, PageTableManager};
use crate::sync::Spinlock;
use crate::types::{PhysAddr, VirtAddr};

/// Virtual memory area descriptor
#[derive(Debug, Clone)]
struct VmallocArea {
	start: VirtAddr,
	end: VirtAddr,
	size: usize,
	pages: alloc::vec::Vec<PhysAddr>,
}

/// Vmalloc allocator
struct VmallocAllocator {
	areas: BTreeMap<usize, VmallocArea>,
	next_addr: usize,
	page_table: Option<PageTableManager>,
}

impl VmallocAllocator {
	const fn new() -> Self {
		Self {
			areas: BTreeMap::new(),
			next_addr: 0xFFFF_8000_0000_0000, // Kernel vmalloc area start
			page_table: None,
		}
	}

	fn init(&mut self) -> Result<()> {
		self.page_table = Some(PageTableManager::new()?);
		Ok(())
	}

	fn allocate(&mut self, size: usize) -> Result<VirtAddr> {
		if size == 0 {
			return Err(Error::InvalidArgument);
		}

		// Align size to page boundary
		let aligned_size = (size + 4095) & !4095;
		let pages_needed = aligned_size / 4096;

		// Find virtual address space
		let start_addr = self.find_free_area(aligned_size)?;
		let end_addr = start_addr + aligned_size;

		// Allocate physical pages
		let mut pages = alloc::vec::Vec::new();
		for _ in 0..pages_needed {
			let pfn = alloc_pages(0, GfpFlags::KERNEL)?;
			pages.push(pfn.to_phys_addr());
		}

		// Map virtual to physical pages
		if let Some(ref mut page_table) = self.page_table {
			for (i, &phys_addr) in pages.iter().enumerate() {
				let virt_addr = VirtAddr::new(start_addr + i * 4096);
				page_table.map_page(
					virt_addr,
					phys_addr,
					PageTableFlags::kernel_page(),
				)?;
			}
		}

		let area = VmallocArea {
			start: VirtAddr::new(start_addr),
			end: VirtAddr::new(end_addr),
			size: aligned_size,
			pages,
		};

		self.areas.insert(start_addr, area);
		Ok(VirtAddr::new(start_addr))
	}

	fn deallocate(&mut self, addr: VirtAddr) -> Result<()> {
		let addr_usize = addr.as_usize();

		if let Some(area) = self.areas.remove(&addr_usize) {
			// Unmap pages from page tables
			if let Some(ref mut page_table) = self.page_table {
				for i in 0..(area.size / 4096) {
					let virt_addr =
						VirtAddr::new(area.start.as_usize() + i * 4096);
					let _ = page_table.unmap_page(virt_addr);
				}
			}

			// Free physical pages
			for phys_addr in area.pages {
				if let Some(_page_ptr) = NonNull::new(
					phys_addr.as_usize() as *mut crate::memory::Page
				) {
					let pfn = PageFrameNumber::from_phys_addr(phys_addr);
					free_pages(pfn, 0);
				}
			}

			Ok(())
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn find_free_area(&mut self, size: usize) -> Result<usize> {
		// Simple linear search for free area
		// In a real implementation, this would be more sophisticated
		let mut addr = self.next_addr;

		// Check if area is free
		for (start, area) in &self.areas {
			if addr >= *start && addr < area.end.as_usize() {
				addr = area.end.as_usize();
			}
		}

		self.next_addr = addr + size;
		Ok(addr)
	}

	pub fn stats(&self) -> (usize, usize) {
		let mut allocated_bytes = 0;
		for (_, area) in &self.areas {
			allocated_bytes += area.size;
		}
		(self.areas.len(), allocated_bytes)
	}
}

static VMALLOC_ALLOCATOR: Spinlock<VmallocAllocator> = Spinlock::new(VmallocAllocator::new());

/// Get vmalloc statistics
pub fn get_stats() -> (usize, usize) {
	let allocator = VMALLOC_ALLOCATOR.lock();
	allocator.stats()
}

/// Allocate virtual memory
pub fn vmalloc(size: usize) -> Result<VirtAddr> {
	let mut allocator = VMALLOC_ALLOCATOR.lock();
	allocator.allocate(size)
}

/// Free virtual memory
pub fn vfree(addr: VirtAddr) {
	let mut allocator = VMALLOC_ALLOCATOR.lock();
	let _ = allocator.deallocate(addr);
}

/// Allocate zeroed virtual memory
pub fn vzalloc(size: usize) -> Result<VirtAddr> {
	let addr = vmalloc(size)?;

	// Zero the memory
	unsafe {
		core::ptr::write_bytes(addr.as_usize() as *mut u8, 0, size);
	}

	Ok(addr)
}

/// Map physical memory into virtual space
pub fn vmap_phys(phys_addr: PhysAddr, size: usize) -> Result<VirtAddr> {
	let start_addr;
	let aligned_size;
	{
		let mut allocator = VMALLOC_ALLOCATOR.lock();
		if allocator.page_table.is_none() {
			return Err(Error::NotInitialized);
		}
		aligned_size = (size + 4095) & !4095;
		start_addr = allocator.find_free_area(aligned_size)?;
	}

	let mut allocator = VMALLOC_ALLOCATOR.lock();
	let page_table = allocator.page_table.as_mut().unwrap();

	// Map virtual to physical pages
	let pages_needed = aligned_size / 4096;
	for i in 0..pages_needed {
		let virt_addr = VirtAddr::new(start_addr + i * 4096);
		let phys_addr = PhysAddr::new(phys_addr.as_usize() + i * 4096);
		page_table.map_page(
			virt_addr,
			phys_addr,
			PageTableFlags::kernel_page() | PageTableFlags::NO_EXECUTE,
		)?;
	}

	let end_addr = start_addr + aligned_size;
	let area = VmallocArea {
		start: VirtAddr::new(start_addr),
		end: VirtAddr::new(end_addr),
		size: aligned_size,
		pages: alloc::vec![], // We don't own these pages
	};

	allocator.areas.insert(start_addr, area);
	Ok(VirtAddr::new(start_addr))
}

pub fn vmap(pages: &[PhysAddr], count: usize) -> Result<VirtAddr> {
	let size = count * 4096;
	let mut allocator = VMALLOC_ALLOCATOR.lock();

	// Find virtual address space
	let start_addr = allocator.find_free_area(size)?;

	let area = VmallocArea {
		start: VirtAddr::new(start_addr),
		end: VirtAddr::new(start_addr + size),
		size,
		pages: pages.to_vec(),
	};

	allocator.areas.insert(start_addr, area);

	// TODO: Set up page table mappings

	Ok(VirtAddr::new(start_addr))
}

/// Unmap virtual memory
pub fn vunmap(addr: VirtAddr) {
	vfree(addr);
}

/// Initialize vmalloc allocator
pub fn init() -> Result<()> {
	let mut allocator = VMALLOC_ALLOCATOR.lock();
	allocator.init()
}
