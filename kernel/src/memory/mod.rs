// SPDX-License-Identifier: GPL-2.0

//! Memory management subsystem

pub mod allocator;
pub mod page;
pub mod vmalloc;
pub mod kmalloc;

use crate::types::{PhysAddr, VirtAddr, Pfn};
use crate::error::{Error, Result};
use core::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;

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

/// Global heap allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

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

/// Initialize the memory management subsystem with proper Linux-style initialization
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
        total_pages: 0,  // TODO: implement
        free_pages: 0,
        used_pages: 0,
        kernel_pages: 0,
    }
}

/// Allocate a page of physical memory
pub fn alloc_page() -> Result<PhysAddr> {
    page::alloc_page()
}

/// Free a page of physical memory
pub fn free_page(addr: PhysAddr) {
    page::free_page(addr)
}

/// Map a virtual address to a physical address
pub fn map_page(virt: VirtAddr, phys: PhysAddr) -> Result<()> {
    // TODO: implement page table mapping
    Ok(())
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
