// SPDX-License-Identifier: GPL-2.0

//! Page table management for x86_64

use core::arch::asm;

use crate::error::{Error, Result};
use crate::memory::allocator::{alloc_pages, free_pages, GfpFlags, PageFrameNumber};
use crate::types::{PhysAddr, VirtAddr, PAGE_SIZE};

/// Page table entry flags
#[derive(Debug, Clone, Copy)]
pub struct PageTableFlags(pub u64);

impl PageTableFlags {
	pub const PRESENT: Self = Self(1 << 0);
	pub const WRITABLE: Self = Self(1 << 1);
	pub const USER_ACCESSIBLE: Self = Self(1 << 2);
	pub const WRITE_THROUGH: Self = Self(1 << 3);
	pub const NO_CACHE: Self = Self(1 << 4);
	pub const ACCESSED: Self = Self(1 << 5);
	pub const DIRTY: Self = Self(1 << 6);
	pub const HUGE_PAGE: Self = Self(1 << 7);
	pub const GLOBAL: Self = Self(1 << 8);
	pub const NO_EXECUTE: Self = Self(1 << 63);

	pub fn empty() -> Self {
		Self(0)
	}

	pub fn kernel_page() -> Self {
		Self::PRESENT | Self::WRITABLE
	}

	pub fn user_page() -> Self {
		Self::PRESENT | Self::WRITABLE | Self::USER_ACCESSIBLE
	}

	pub fn contains(self, flag: Self) -> bool {
		self.0 & flag.0 != 0
	}
}

impl core::ops::BitOr for PageTableFlags {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
}

impl core::ops::BitOrAssign for PageTableFlags {
	fn bitor_assign(&mut self, rhs: Self) {
		self.0 |= rhs.0;
	}
}

/// Page table entry
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
	pub fn new() -> Self {
		Self(0)
	}

	pub fn is_present(self) -> bool {
		self.0 & 1 != 0
	}

	pub fn set_frame(self, frame: PageFrameNumber, flags: PageTableFlags) -> Self {
		let addr = frame.to_phys_addr().as_usize() as u64;
		Self((addr & !0xfff) | flags.0)
	}

	pub fn frame(self) -> Option<PageFrameNumber> {
		if self.is_present() {
			Some(PageFrameNumber::from_phys_addr(PhysAddr::new(
				(self.0 & !0xfff) as usize,
			)))
		} else {
			None
		}
	}

	pub fn flags(self) -> PageTableFlags {
		PageTableFlags(self.0 & 0xfff)
	}
}

/// Page table with 512 entries (x86_64)
#[repr(align(4096))]
pub struct PageTable {
	entries: [PageTableEntry; 512],
}

impl PageTable {
	pub fn new() -> Self {
		Self {
			entries: [PageTableEntry::new(); 512],
		}
	}

	pub fn zero(&mut self) {
		for entry in &mut self.entries {
			*entry = PageTableEntry::new();
		}
	}

	pub fn entry(&mut self, index: usize) -> &mut PageTableEntry {
		&mut self.entries[index]
	}

	pub fn entry_ref(&self, index: usize) -> &PageTableEntry {
		&self.entries[index]
	}
}

/// Page table manager
pub struct PageTableManager {
	root_table: PhysAddr,
}

impl PageTableManager {
	pub fn new() -> Result<Self> {
		// Allocate a page for the root page table (PML4)
		let pfn = alloc_pages(0, GfpFlags::KERNEL)?;
		let root_table = pfn.to_phys_addr();

		// Zero the root table
		unsafe {
			let table = root_table.as_usize() as *mut PageTable;
			(*table).zero();
		}

		Ok(Self { root_table })
	}

	pub fn root_table_addr(&self) -> PhysAddr {
		self.root_table
	}

	/// Map a virtual page to a physical page
	pub fn map_page(
		&mut self,
		virt_addr: VirtAddr,
		phys_addr: PhysAddr,
		flags: PageTableFlags,
	) -> Result<()> {
		let virt_page = virt_addr.as_usize() / PAGE_SIZE;
		let pfn = PageFrameNumber::from_phys_addr(phys_addr);

		// Extract page table indices from virtual address
		let pml4_index = (virt_page >> 27) & 0x1ff;
		let pdp_index = (virt_page >> 18) & 0x1ff;
		let pd_index = (virt_page >> 9) & 0x1ff;
		let pt_index = virt_page & 0x1ff;

		// Walk and create page tables as needed
		let pml4 = unsafe { &mut *(self.root_table.as_usize() as *mut PageTable) };

		// Get or create PDP
		let pdp_addr = if pml4.entry_ref(pml4_index).is_present() {
			pml4.entry_ref(pml4_index).frame().unwrap().to_phys_addr()
		} else {
			let pdp_pfn = alloc_pages(0, GfpFlags::KERNEL)?;
			let pdp_addr = pdp_pfn.to_phys_addr();
			unsafe {
				let pdp_table = pdp_addr.as_usize() as *mut PageTable;
				(*pdp_table).zero();
			}
			*pml4.entry(pml4_index) = PageTableEntry::new()
				.set_frame(pdp_pfn, PageTableFlags::kernel_page());
			pdp_addr
		};

		// Get or create PD
		let pdp = unsafe { &mut *(pdp_addr.as_usize() as *mut PageTable) };
		let pd_addr = if pdp.entry_ref(pdp_index).is_present() {
			pdp.entry_ref(pdp_index).frame().unwrap().to_phys_addr()
		} else {
			let pd_pfn = alloc_pages(0, GfpFlags::KERNEL)?;
			let pd_addr = pd_pfn.to_phys_addr();
			unsafe {
				let pd_table = pd_addr.as_usize() as *mut PageTable;
				(*pd_table).zero();
			}
			*pdp.entry(pdp_index) = PageTableEntry::new()
				.set_frame(pd_pfn, PageTableFlags::kernel_page());
			pd_addr
		};

		// Get or create PT
		let pd = unsafe { &mut *(pd_addr.as_usize() as *mut PageTable) };
		let pt_addr = if pd.entry_ref(pd_index).is_present() {
			pd.entry_ref(pd_index).frame().unwrap().to_phys_addr()
		} else {
			let pt_pfn = alloc_pages(0, GfpFlags::KERNEL)?;
			let pt_addr = pt_pfn.to_phys_addr();
			unsafe {
				let pt_table = pt_addr.as_usize() as *mut PageTable;
				(*pt_table).zero();
			}
			*pd.entry(pd_index) = PageTableEntry::new()
				.set_frame(pt_pfn, PageTableFlags::kernel_page());
			pt_addr
		};

		// Set the final page mapping
		let pt = unsafe { &mut *(pt_addr.as_usize() as *mut PageTable) };
		*pt.entry(pt_index) = PageTableEntry::new().set_frame(pfn, flags);

		// Flush TLB for this page
		unsafe {
			asm!("invlpg [{}]", in(reg) virt_addr.as_usize(), options(nostack, preserves_flags));
		}

		Ok(())
	}

	/// Unmap a virtual page
	pub fn unmap_page(&mut self, virt_addr: VirtAddr) -> Result<()> {
		let virt_page = virt_addr.as_usize() / PAGE_SIZE;

		// Extract page table indices
		let pml4_index = (virt_page >> 27) & 0x1ff;
		let pdp_index = (virt_page >> 18) & 0x1ff;
		let pd_index = (virt_page >> 9) & 0x1ff;
		let pt_index = virt_page & 0x1ff;

		// Walk page tables
		let pml4 = unsafe { &mut *(self.root_table.as_usize() as *mut PageTable) };

		if !pml4.entry_ref(pml4_index).is_present() {
			return Err(Error::InvalidArgument);
		}

		let pdp_addr = pml4.entry_ref(pml4_index).frame().unwrap().to_phys_addr();
		let pdp = unsafe { &mut *(pdp_addr.as_usize() as *mut PageTable) };

		if !pdp.entry_ref(pdp_index).is_present() {
			return Err(Error::InvalidArgument);
		}

		let pd_addr = pdp.entry_ref(pdp_index).frame().unwrap().to_phys_addr();
		let pd = unsafe { &mut *(pd_addr.as_usize() as *mut PageTable) };

		if !pd.entry_ref(pd_index).is_present() {
			return Err(Error::InvalidArgument);
		}

		let pt_addr = pd.entry_ref(pd_index).frame().unwrap().to_phys_addr();
		let pt = unsafe { &mut *(pt_addr.as_usize() as *mut PageTable) };

		// Clear the page table entry
		*pt.entry(pt_index) = PageTableEntry::new();

		// Flush TLB for this page
		unsafe {
			asm!("invlpg [{}]", in(reg) virt_addr.as_usize(), options(nostack, preserves_flags));
		}

		Ok(())
	}

	/// Switch to this page table
	pub fn switch_to(&self) {
		unsafe {
			asm!("mov cr3, {}", in(reg) self.root_table.as_usize(), options(nostack, preserves_flags));
		}
	}
}
