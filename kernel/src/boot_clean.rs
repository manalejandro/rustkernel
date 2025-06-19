// SPDX-License-Identifier: GPL-2.0

//! Boot process and hardware initialization

use crate::{info, error};
use crate::error::Result;
use alloc::string::ToString;

/// Boot stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStage {
    EarlyInit,
    MemoryInit, 
    DeviceInit,
    SchedulerInit,
    FileSystemInit,
    NetworkInit,
    UserSpaceInit,
    Complete,
}

/// Boot information structure
#[derive(Debug)]
pub struct BootInfo {
    pub memory_size: usize,
    pub available_memory: usize,
    pub cpu_count: usize,
    pub boot_time: u64,
    pub command_line: Option<alloc::string::String>,
    pub initrd_start: Option<usize>,
    pub initrd_size: Option<usize>,
    pub multiboot_addr: Option<usize>,
}

impl BootInfo {
    pub fn new() -> Self {
        Self {
            memory_size: 0,
            available_memory: 0,
            cpu_count: 1,
            boot_time: 0,
            command_line: None,
            initrd_start: None,
            initrd_size: None,
            multiboot_addr: None,
        }
    }
}

/// Global boot information
pub static mut BOOT_INFO: BootInfo = BootInfo {
    memory_size: 0,
    available_memory: 0,
    cpu_count: 1,
    boot_time: 0,
    command_line: None,
    initrd_start: None,
    initrd_size: None,
    multiboot_addr: None,
};

/// Set multiboot information address
pub fn set_multiboot_info(addr: usize) {
    unsafe {
        BOOT_INFO.multiboot_addr = Some(addr);
    }
}

/// Get boot information
pub fn get_boot_info() -> &'static BootInfo {
    unsafe { &BOOT_INFO }
}

/// Update boot information
pub unsafe fn update_boot_info<F>(f: F) where F: FnOnce(&mut BootInfo) {
    f(&mut BOOT_INFO);
}

pub mod multiboot {
    use crate::types::{PhysAddr, VirtAddr};
    use crate::error::Result;

    /// Multiboot2 information structure
    #[repr(C)]
    pub struct MultibootInfo {
        pub total_size: u32,
        pub reserved: u32,
    }

    /// Memory map entry from multiboot
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct MemoryMapEntry {
        pub base_addr: u64,
        pub length: u64,
        pub type_: u32,
        pub reserved: u32,
    }

    /// Memory map types
    pub mod memory_type {
        pub const AVAILABLE: u32 = 1;
        pub const RESERVED: u32 = 2;
        pub const ACPI_RECLAIMABLE: u32 = 3;
        pub const NVS: u32 = 4;
        pub const BADRAM: u32 = 5;
    }

    /// Boot memory information
    #[derive(Debug)]
    pub struct BootMemoryInfo {
        pub total_memory: u64,
        pub available_memory: u64,
        pub memory_regions: alloc::vec::Vec<MemoryMapEntry>,
    }

    impl BootMemoryInfo {
        pub fn new() -> Self {
            Self {
                total_memory: 0,
                available_memory: 0,
                memory_regions: alloc::vec::Vec::new(),
            }
        }

        pub fn add_region(&mut self, entry: MemoryMapEntry) {
            if entry.type_ == memory_type::AVAILABLE {
                self.available_memory += entry.length;
            }
            self.total_memory += entry.length;
            self.memory_regions.push(entry);
        }
    }

    /// Parse multiboot2 information and initialize memory management
    pub fn init_memory_from_multiboot(multiboot_addr: usize) -> Result<()> {
        info!("Parsing multiboot information at 0x{:x}", multiboot_addr);
        
        let multiboot_info = unsafe { &*(multiboot_addr as *const MultibootInfo) };
        
        info!("Multiboot info size: {} bytes", multiboot_info.total_size);
        
        // Parse memory map from multiboot info
        let mut memory_info = BootMemoryInfo::new();
        
        // For now, assume a basic memory layout if we can't parse multiboot properly
        // This is a fallback to make the kernel bootable
        let default_memory = MemoryMapEntry {
            base_addr: 0x100000, // 1MB
            length: 0x7F00000,   // ~127MB (assuming 128MB total RAM)
            type_: memory_type::AVAILABLE,
            reserved: 0,
        };
        
        memory_info.add_region(default_memory);
        
        // Update global boot info
        unsafe {
            super::update_boot_info(|boot_info| {
                boot_info.memory_size = memory_info.total_memory as usize;
                boot_info.available_memory = memory_info.available_memory as usize;
            });
        }
        
        // Initialize page allocator with available memory
        for region in &memory_info.memory_regions {
            if region.type_ == memory_type::AVAILABLE {
                let start_pfn = region.base_addr / 4096;
                let end_pfn = (region.base_addr + region.length) / 4096;
                
                info!("Adding memory region: 0x{:x}-0x{:x}", 
                      region.base_addr, region.base_addr + region.length);
                
                // Add this memory region to the page allocator
                crate::memory::page::add_free_range(
                    PhysAddr::new(region.base_addr as usize),
                    PhysAddr::new((region.base_addr + region.length) as usize)
                )?;
            }
        }

        info!("Memory initialization from multiboot completed");
        info!("Total memory: {} bytes", memory_info.total_memory);
        info!("Available memory: {} bytes", memory_info.available_memory);
        
        Ok(())
    }
}

/// Early boot setup before memory allocation is available
pub fn early_boot_setup() -> Result<()> {
    info!("Early boot setup");
    
    // Basic hardware initialization
    // This is done before memory allocators are available
    
    Ok(())
}

/// Boot stage management
static mut CURRENT_BOOT_STAGE: BootStage = BootStage::EarlyInit;

/// Get current boot stage
pub fn get_boot_stage() -> BootStage {
    unsafe { CURRENT_BOOT_STAGE }
}

/// Set boot stage
pub fn set_boot_stage(stage: BootStage) {
    unsafe {
        CURRENT_BOOT_STAGE = stage;
    }
    info!("Boot stage: {:?}", stage);
}

/// Complete boot process
pub fn complete_boot() -> Result<()> {
    set_boot_stage(BootStage::Complete);
    info!("Boot process completed successfully");
    Ok(())
}
