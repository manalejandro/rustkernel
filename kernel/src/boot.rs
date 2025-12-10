// SPDX-License-Identifier: GPL-2.0

//! Boot process and hardware initialization

use alloc::string::ToString;

use crate::error::Result;
use crate::{error, info};

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
pub unsafe fn update_boot_info<F>(f: F)
where
	F: FnOnce(&mut BootInfo),
{
	f(&mut BOOT_INFO);
}

pub mod multiboot {
	use crate::error::Result;
	use crate::info;
	use crate::types::{PhysAddr, VirtAddr};

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
		pub memory_regions: [MemoryMapEntry; 32],
		pub region_count: usize,
	}

	impl BootMemoryInfo {
		pub fn new() -> Self {
			Self {
				total_memory: 0,
				available_memory: 0,
				memory_regions: [MemoryMapEntry {
					base_addr: 0,
					length: 0,
					type_: 0,
					reserved: 0,
				}; 32],
				region_count: 0,
			}
		}

		pub fn add_region(&mut self, entry: MemoryMapEntry) {
			if entry.type_ == memory_type::AVAILABLE {
				self.available_memory += entry.length;
			}
			self.total_memory += entry.length;
			if self.region_count < 32 {
				self.memory_regions[self.region_count] = entry;
				self.region_count += 1;
			}
		}
	}

	/// Parse multiboot2 information and initialize memory management
	pub fn init_memory_from_multiboot(multiboot_addr: usize) -> Result<()> {
		crate::console::write_str("Parsing multiboot\n");

		// Validate multiboot address is in identity-mapped range (0-1GB)
		if multiboot_addr >= 0x40000000 {
			// 1GB
			crate::console::write_str("ERROR: multiboot addr out of range\n");
			return Err(crate::error::Error::InvalidArgument);
		}

		crate::console::write_str("Multiboot addr validated\n");

		let multiboot_info = unsafe { &*(multiboot_addr as *const MultibootInfo) };

		crate::console::write_str("Got multiboot info\n");

		// Parse memory map from multiboot info
		let mut memory_info = BootMemoryInfo::new();

		crate::console::write_str("Created BootMemoryInfo\n");

		// For now, assume a basic memory layout if we can't parse multiboot properly
		// This is a fallback to make the kernel bootable
		let default_memory = MemoryMapEntry {
			base_addr: 0x100000, // 1MB
			length: 0x7F00000,   // ~127MB (assuming 128MB total RAM)
			type_: memory_type::AVAILABLE,
			reserved: 0,
		};

		crate::console::write_str("Adding default memory region\n");
		memory_info.add_region(default_memory);

		// Update global boot info
		unsafe {
			super::update_boot_info(|boot_info| {
				boot_info.memory_size = memory_info.total_memory as usize;
				boot_info.available_memory = memory_info.available_memory as usize;
			});
		}

		// Initialize page allocator with available memory
		// Note: Only first 1GB is identity-mapped in boot.s
		const MAX_IDENTITY_MAPPED: u64 = 1024 * 1024 * 1024; // 1GB

		crate::console::write_str("Processing memory regions\n");

		for i in 0..memory_info.region_count {
			crate::console::write_str("Region loop iteration\n");
			let region = &memory_info.memory_regions[i];

			if region.type_ == memory_type::AVAILABLE {
				let start_addr = region.base_addr;
				let end_addr = region.base_addr + region.length;

				crate::console::write_str("Available region found\n");

				// Clamp to identity-mapped region
				let safe_start = start_addr.max(0x100000); // Skip first 1MB (BIOS/kernel)
				let safe_end = end_addr.min(MAX_IDENTITY_MAPPED);

				crate::console::write_str("Clamped region\n");

				if safe_start >= safe_end {
					crate::console::write_str("Skipping invalid range\n");
					continue; // Skip invalid/unmapped region
				}

				crate::console::write_str("About to call add_free_range\n");
				// Add this memory region to the page allocator
				crate::memory::page::add_free_range(
					PhysAddr::new(safe_start as usize),
					PhysAddr::new(safe_end as usize),
				)?;
				crate::console::write_str("Successfully added free range\n");
			}
		}

		crate::console::write_str("Memory init completed\n");

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

/// Initialize multiboot information
/// This should be called at the very beginning of kernel execution
pub fn multiboot_init() {
	// Parse multiboot information from bootloader
	// For now, we'll use a combination of detection and defaults

	let detected_memory = detect_memory_size();
	let cpu_count = detect_cpu_count();

	unsafe {
		BOOT_INFO = BootInfo {
			memory_size: detected_memory,
			available_memory: (detected_memory * 95) / 100, // 95% available
			cpu_count,
			boot_time: read_tsc(),
			command_line: None,
			initrd_start: None,
			initrd_size: None,
			multiboot_addr: None,
		};
	}

	info!("Multiboot information initialized");
	info!("  Memory size: {} MB", detected_memory / (1024 * 1024));
	info!(
		"  Available memory: {} MB",
		get_boot_info().available_memory / (1024 * 1024)
	);
	info!("  CPU count: {}", cpu_count);
}

/// Detect total system memory
fn detect_memory_size() -> usize {
	// Use CMOS to get basic memory information
	unsafe {
		// Read extended memory from CMOS (simplified)
		crate::arch::x86_64::port::outb(0x70, 0x17);
		let low = crate::arch::x86_64::port::inb(0x71) as usize;
		crate::arch::x86_64::port::outb(0x70, 0x18);
		let high = crate::arch::x86_64::port::inb(0x71) as usize;

		let extended_mem = (high << 8) | low; // in KB
		let total_mem = 1024 * 1024 + (extended_mem * 1024); // Base 1MB + extended

		// Reasonable bounds checking
		if total_mem < 16 * 1024 * 1024 {
			// Default to 64MB if detection seems wrong
			64 * 1024 * 1024
		} else if total_mem > 8 * 1024 * 1024 * 1024 {
			// Cap at 8GB for safety
			8 * 1024 * 1024 * 1024
		} else {
			total_mem
		}
	}
}

/// Detect CPU count (simplified)
fn detect_cpu_count() -> usize {
	// For now, assume single CPU
	// In a real implementation, this would parse ACPI tables or use CPUID
	1
}

/// Read Time Stamp Counter
fn read_tsc() -> u64 {
	unsafe {
		let low: u32;
		let high: u32;
		core::arch::asm!(
		    "rdtsc",
		    out("eax") low,
		    out("edx") high,
		    options(nomem, nostack, preserves_flags)
		);
		((high as u64) << 32) | (low as u64)
	}
}
