// SPDX-License-Identifier: GPL-2.0

//! Memory management subsystem

pub mod advanced_allocator;
pub mod allocator;
pub mod kmalloc;
pub mod page;
pub mod page_table;
pub mod vmalloc;

// Re-export important types
use alloc::string::String;

use linked_list_allocator::LockedHeap;
pub use page::Page;

use crate::error::{Error, Result};
pub use crate::types::{Pfn, PhysAddr, VirtAddr}; // Re-export from types

/// GFP (Get Free Pages) flags - compatible with Linux kernel
pub mod gfp {
	pub const GFP_KERNEL: u32 = 0;
	pub const GFP_ATOMIC: u32 = 1;
	pub const GFP_USER: u32 = 2;
	pub const GFP_HIGHUSER: u32 = 3;
	pub const GFP_DMA: u32 = 4;
	pub const GFP_DMA32: u32 = 8;
	pub const GFP_NOWAIT: u32 = 16;
	pub const GFP_NOIO: u32 = 32;
	pub const GFP_NOFS: u32 = 64;
	pub const GFP_ZERO: u32 = 128;
}

/// Global heap allocator (using advanced allocator)
// #[global_allocator]
// static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Linux-compatible allocation flags
#[derive(Clone, Copy, PartialEq)]
pub struct AllocFlags(u32);

impl AllocFlags {
	pub const fn new(flags: u32) -> Self {
		Self(flags)
	}

	pub fn as_raw(self) -> u32 {
		self.0
	}

	pub fn contains(self, flags: AllocFlags) -> bool {
		(self.0 & flags.0) == flags.0
	}
}

/// GFP flags constants
pub const GFP_KERNEL: AllocFlags = AllocFlags::new(gfp::GFP_KERNEL);
pub const GFP_ATOMIC: AllocFlags = AllocFlags::new(gfp::GFP_ATOMIC);
pub const GFP_USER: AllocFlags = AllocFlags::new(gfp::GFP_USER);

/// Page mapping flags
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PageFlags(u64);

impl PageFlags {
	pub const PRESENT: PageFlags = PageFlags(1 << 0);
	pub const WRITABLE: PageFlags = PageFlags(1 << 1);
	pub const USER: PageFlags = PageFlags(1 << 2);
	pub const EXECUTABLE: PageFlags = PageFlags(1 << 63); // NX bit inverted

	pub const fn new(flags: u64) -> Self {
		Self(flags)
	}

	pub fn as_raw(self) -> u64 {
		self.0
	}

	pub fn contains(self, flags: PageFlags) -> bool {
		(self.0 & flags.0) == flags.0
	}
}

impl core::ops::BitOr for PageFlags {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		PageFlags(self.0 | rhs.0)
	}
}

/// Initialize the memory management subsystem with proper Linux-style
/// initialization
pub fn init() -> Result<()> {
	allocator::init()?;
	page::init()?;

	// Initialize zone allocator
	init_zones()?;

	// Set up buddy allocator
	init_buddy_allocator()?;

	// Initialize slab allocator
	init_slab_allocator()?;

	crate::info!("Memory management initialized");
	Ok(())
}

/// Initialize memory zones (DMA, Normal, High)
fn init_zones() -> Result<()> {
	// TODO: Set up memory zones based on architecture
	Ok(())
}

/// Initialize buddy allocator for page allocation
fn init_buddy_allocator() -> Result<()> {
	// TODO: Set up buddy allocator
	Ok(())
}

/// Initialize slab allocator for object caching
fn init_slab_allocator() -> Result<()> {
	// TODO: Set up SLAB/SLUB allocator
	Ok(())
}

/// Physical memory information
#[derive(Debug)]
pub struct MemoryInfo {
	pub total_pages: usize,
	pub free_pages: usize,
	pub used_pages: usize,
	pub kernel_pages: usize,
}

/// Get current memory information
pub fn memory_info() -> MemoryInfo {
	MemoryInfo {
		total_pages: 0, // TODO: implement
		free_pages: 0,
		used_pages: 0,
		kernel_pages: 0,
	}
}

/// Memory statistics for diagnostics
#[derive(Debug, Clone)]
pub struct MemoryStats {
	pub total: usize,
	pub used: usize,
	pub free: usize,
	pub usage_percent: usize,
}

/// Get memory statistics for diagnostics
pub fn get_memory_stats() -> Result<MemoryStats> {
	let info = memory_info();
	let total = if info.total_pages > 0 {
		info.total_pages * 4096
	} else {
		64 * 1024 * 1024
	}; // Default 64MB
	let used = info.used_pages * 4096;
	let free = total - used;
	let usage_percent = if total > 0 { (used * 100) / total } else { 0 };

	Ok(MemoryStats {
		total,
		used,
		free,
		usage_percent,
	})
}

/// Allocate a page of physical memory
pub fn alloc_page() -> Result<PhysAddr> {
	page::alloc_page()
}

/// Free a page of physical memory
pub fn free_page(addr: PhysAddr) {
	page::free_page(addr)
}

/// Allocate a page of physical memory
pub fn allocate_page() -> Result<PhysAddr> {
	page::allocate_page()
}

/// Map a virtual address to a physical address
pub fn map_page(virt: VirtAddr, phys: PhysAddr, flags: PageFlags) -> Result<()> {
	// TODO: implement page table mapping with flags
	Ok(())
}

/// Map a virtual address to a physical address (simple version)
pub fn map_page_simple(virt: VirtAddr, phys: PhysAddr) -> Result<()> {
	// TODO: implement page table mapping
	map_page(virt, phys, PageFlags::PRESENT | PageFlags::WRITABLE)
}

/// Unmap a virtual address
pub fn unmap_page(virt: VirtAddr) -> Result<()> {
	// TODO: implement page table unmapping
	Ok(())
}

/// Convert virtual address to physical address
pub fn virt_to_phys(virt: VirtAddr) -> Result<PhysAddr> {
	// TODO: implement address translation
	Ok(PhysAddr::new(virt.as_usize()))
}

/// Convert physical address to virtual address
pub fn phys_to_virt(phys: PhysAddr) -> Result<VirtAddr> {
	// TODO: implement address translation
	Ok(VirtAddr::new(phys.as_usize()))
}

/// Page table entry
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
	pub const fn new() -> Self {
		Self(0)
	}

	pub fn present(self) -> bool {
		self.0 & 1 != 0
	}

	pub fn writable(self) -> bool {
		self.0 & 2 != 0
	}

	pub fn user_accessible(self) -> bool {
		self.0 & 4 != 0
	}

	pub fn frame(self) -> Pfn {
		Pfn((self.0 >> 12) as usize)
	}

	pub fn set_present(&mut self, present: bool) {
		if present {
			self.0 |= 1;
		} else {
			self.0 &= !1;
		}
	}

	pub fn set_writable(&mut self, writable: bool) {
		if writable {
			self.0 |= 2;
		} else {
			self.0 &= !2;
		}
	}

	pub fn set_user_accessible(&mut self, user: bool) {
		if user {
			self.0 |= 4;
		} else {
			self.0 &= !4;
		}
	}

	pub fn set_frame(&mut self, frame: Pfn) {
		self.0 = (self.0 & 0xfff) | ((frame.0 as u64) << 12);
	}
}

/// Page table
#[repr(align(4096))]
pub struct PageTable {
	entries: [PageTableEntry; 512],
}

impl PageTable {
	pub const fn new() -> Self {
		Self {
			entries: [PageTableEntry::new(); 512],
		}
	}

	pub fn zero(&mut self) {
		for entry in self.entries.iter_mut() {
			*entry = PageTableEntry::new();
		}
	}
}

/// Memory mapping flags
bitflags::bitflags! {
    pub struct MapFlags: u32 {
	const READ = 1 << 0;
	const WRITE = 1 << 1;
	const EXECUTE = 1 << 2;
	const USER = 1 << 3;
	const GLOBAL = 1 << 4;
	const CACHED = 1 << 5;
	const DEVICE = 1 << 6;
    }
}

/// User space pointer wrapper for safe kernel-user space data transfer
#[derive(Debug, Clone, Copy)]
pub struct UserPtr<T> {
	ptr: *mut T,
}

impl<T> UserPtr<T> {
	/// Create a new UserPtr with validation
	pub fn new(ptr: *mut T) -> Result<Self> {
		if ptr.is_null() {
			return Err(Error::InvalidArgument);
		}
		// TODO: Add proper user space validation
		Ok(Self { ptr })
	}

	/// Create a new UserPtr from const pointer
	pub fn from_const(ptr: *const T) -> Result<Self> {
		Self::new(ptr as *mut T)
	}

	/// Get the raw pointer
	pub fn as_ptr(&self) -> *mut T {
		self.ptr
	}

	/// Cast to different type
	pub fn cast<U>(&self) -> UserPtr<U> {
		UserPtr {
			ptr: self.ptr as *mut U,
		}
	}

	/// Check if the pointer is null
	pub fn is_null(&self) -> bool {
		self.ptr.is_null()
	}

	/// Write data to user space
	pub fn write(&self, data: T) -> Result<()> {
		// TODO: Implement proper user space access validation
		// For now, this is a stub
		if self.ptr.is_null() {
			return Err(Error::InvalidArgument);
		}

		// In a real kernel, this would use copy_to_user or similar
		// For now, we'll use unsafe direct write (this is NOT safe for real use)
		unsafe {
			core::ptr::write(self.ptr, data);
		}
		Ok(())
	}
}

/// User space slice pointer for array-like data
#[derive(Debug, Clone, Copy)]
pub struct UserSlicePtr {
	ptr: *mut u8,
	len: usize,
}

impl UserSlicePtr {
	/// Create a new UserSlicePtr (unsafe as it's not validated)
	pub unsafe fn new(ptr: *mut u8, len: usize) -> Self {
		Self { ptr, len }
	}

	/// Get the raw pointer
	pub fn as_ptr(&self) -> *mut u8 {
		self.ptr
	}

	/// Get the length
	pub fn len(&self) -> usize {
		self.len
	}

	/// Check if empty
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	/// Copy data from a slice to user space
	pub fn copy_from_slice(&self, data: &[u8]) -> Result<()> {
		// TODO: Implement proper user space access validation
		// For now, this is a stub
		if self.ptr.is_null() {
			return Err(Error::InvalidArgument);
		}

		let copy_len = core::cmp::min(self.len, data.len());

		// In a real kernel, this would use copy_to_user or similar
		// For now, we'll use unsafe direct copy (this is NOT safe for real use)
		unsafe {
			core::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, copy_len);
		}
		Ok(())
	}

	/// Copy data from user space to a slice
	pub fn copy_to_slice(&self, data: &mut [u8]) -> Result<()> {
		// TODO: Implement proper user space access validation
		// For now, this is a stub
		if self.ptr.is_null() {
			return Err(Error::InvalidArgument);
		}

		let copy_len = core::cmp::min(self.len, data.len());

		// In a real kernel, this would use copy_from_user or similar
		// For now, we'll use unsafe direct copy (this is NOT safe for real use)
		unsafe {
			core::ptr::copy_nonoverlapping(self.ptr, data.as_mut_ptr(), copy_len);
		}
		Ok(())
	}
}

/// Copy data to user space
pub fn copy_to_user(user_ptr: UserPtr<u8>, data: &[u8]) -> Result<()> {
	// TODO: Implement proper user space access validation
	// This should check if the user pointer is valid and accessible

	if user_ptr.ptr.is_null() {
		return Err(Error::InvalidArgument);
	}

	// In a real kernel, this would use proper copy_to_user with page fault handling
	// For now, we'll use unsafe direct copy (NOT safe for real use)
	unsafe {
		core::ptr::copy_nonoverlapping(data.as_ptr(), user_ptr.ptr, data.len());
	}
	Ok(())
}

/// Copy data from user space
pub fn copy_from_user(data: &mut [u8], user_ptr: UserPtr<u8>) -> Result<()> {
	// TODO: Implement proper user space access validation
	// This should check if the user pointer is valid and accessible

	if user_ptr.ptr.is_null() {
		return Err(Error::InvalidArgument);
	}

	// In a real kernel, this would use proper copy_from_user with page fault
	// handling For now, we'll use unsafe direct copy (NOT safe for real use)
	unsafe {
		core::ptr::copy_nonoverlapping(user_ptr.ptr, data.as_mut_ptr(), data.len());
	}
	Ok(())
}

/// Copy a string from user space
pub fn copy_string_from_user(user_ptr: UserPtr<u8>, max_len: usize) -> Result<String> {
	// TODO: Implement proper user space access validation

	if user_ptr.ptr.is_null() {
		return Err(Error::InvalidArgument);
	}

	let mut buffer = alloc::vec![0u8; max_len];
	let mut len = 0;

	// Copy byte by byte until null terminator or max length
	unsafe {
		for i in 0..max_len {
			let byte = *user_ptr.ptr.add(i);
			if byte == 0 {
				break;
			}
			buffer[i] = byte;
			len += 1;
		}
	}

	buffer.truncate(len);
	String::from_utf8(buffer).map_err(|_| Error::InvalidArgument)
}

/// Memory mapping area structure
#[derive(Debug, Clone)]
pub struct VmaArea {
	pub vm_start: VirtAddr,
	pub vm_end: VirtAddr,
	pub vm_prot: u32,
	pub vm_flags: u32,
}

impl VmaArea {
	pub fn new(start: VirtAddr, end: VirtAddr, prot: u32) -> Self {
		Self {
			vm_start: start,
			vm_end: end,
			vm_prot: prot,
			vm_flags: 0,
		}
	}
}

/// Allocate virtual memory for mmap
pub fn allocate_virtual_memory(size: u64, prot: u32, flags: u32) -> Result<VmaArea> {
	use crate::memory::kmalloc::kmalloc;

	// Allocate physical pages first
	let pages_needed = (size + 4095) / 4096;
	let phys_addr = kmalloc(size as usize)?;

	// Find a free virtual address range
	let virt_addr = find_free_virtual_range(size)?;

	// Map the pages (simplified implementation)
	map_pages(virt_addr, PhysAddr::new(phys_addr as usize), size, prot)?;

	Ok(VmaArea::new(
		virt_addr,
		VirtAddr::new(virt_addr.as_usize() + size as usize),
		prot,
	))
}

/// Free virtual memory
pub fn free_virtual_memory(addr: VirtAddr, size: u64) -> Result<()> {
	// Unmap pages
	unmap_pages(addr, size)?;

	// Free physical memory (simplified)
	crate::memory::kmalloc::kfree(addr.as_usize() as *mut u8);

	Ok(())
}

/// Find a free virtual address range
fn find_free_virtual_range(size: u64) -> Result<VirtAddr> {
	// Simplified implementation - start from user space
	const USER_SPACE_START: usize = 0x400000; // 4MB
	const USER_SPACE_END: usize = 0x80000000; // 2GB

	let mut addr = USER_SPACE_START;
	while addr + size as usize <= USER_SPACE_END {
		// Check if range is free (simplified check)
		if is_virtual_range_free(VirtAddr::new(addr), size) {
			return Ok(VirtAddr::new(addr));
		}
		addr += 4096; // Page size
	}

	Err(Error::ENOMEM)
}

/// Check if virtual range is free
pub fn is_virtual_range_free(_addr: VirtAddr, _size: u64) -> bool {
	// Simplified implementation - assume it's free
	// In a real implementation, this would check page tables
	true
}

/// Map virtual pages to physical pages
fn map_pages(virt: VirtAddr, phys: PhysAddr, size: u64, _prot: u32) -> Result<()> {
	// Simplified page mapping
	// In a real implementation, this would set up page table entries
	crate::info!(
		"Mapping virtual 0x{:x} to physical 0x{:x}, size: {}",
		virt.as_usize(),
		phys.as_usize(),
		size
	);
	Ok(())
}

/// Unmap virtual pages
fn unmap_pages(virt: VirtAddr, size: u64) -> Result<()> {
	// Simplified page unmapping
	crate::info!("Unmapping virtual 0x{:x}, size: {}", virt.as_usize(), size);
	Ok(())
}

/// Heap management for brk syscall
static mut HEAP_START: VirtAddr = VirtAddr::new(0);
static mut HEAP_END: VirtAddr = VirtAddr::new(0);

/// Get current heap end
pub fn get_heap_end() -> VirtAddr {
	unsafe { HEAP_END }
}

/// Set heap end
pub fn set_heap_end(new_end: VirtAddr) -> Result<()> {
	unsafe {
		if HEAP_START.as_usize() == 0 {
			// Initialize heap
			HEAP_START = VirtAddr::new(0x10000000); // 256MB
			HEAP_END = HEAP_START;
		}

		if new_end >= HEAP_START {
			HEAP_END = new_end;
			Ok(())
		} else {
			Err(Error::EINVAL)
		}
	}
}

/// Initialize heap management
pub fn init_heap() -> Result<()> {
	unsafe {
		HEAP_START = VirtAddr::new(0x10000000); // 256MB
		HEAP_END = HEAP_START;
	}
	Ok(())
}
