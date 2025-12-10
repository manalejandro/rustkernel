// SPDX-License-Identifier: GPL-2.0

//! Hardware detection and initialization

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::driver::{PciBar, PciDevice};
use crate::error::Result;
/// CPU Information
#[derive(Debug, Clone)]
pub struct CpuInfo {
	pub vendor: String,
	pub model_name: String,
	pub family: u32,
	pub model: u32,
	pub stepping: u32,
	pub features: Vec<String>,
	pub cache_size: u32,
	pub core_count: u32,
	pub thread_count: u32,
}

/// System Information
#[derive(Debug, Clone)]
pub struct SystemInfo {
	pub cpu: CpuInfo,
	pub total_memory: usize,
	pub available_memory: usize,
	pub boot_device: String,
	pub acpi_available: bool,
	pub pci_devices: Vec<PciDevice>,
}

/// Initialize hardware detection
pub fn init() -> Result<()> {
	crate::info!("Initializing hardware detection...");

	// Detect CPU
	let cpu_info = detect_cpu()?;
	crate::info!("CPU: {} {}", cpu_info.vendor, cpu_info.model_name);
	crate::info!("CPU Cores: {}", cpu_info.core_count);

	// Detect memory
	let memory_info = detect_memory()?;
	crate::info!("Total Memory: {} MB", memory_info / (1024 * 1024));

	// Detect PCI devices
	let pci_devices = detect_pci_devices()?;
	crate::info!("Found {} PCI devices", pci_devices.len());

	Ok(())
}

/// Detect CPU information
pub fn detect_cpu() -> Result<CpuInfo> {
	let mut cpu_info = CpuInfo {
		vendor: String::new(),
		model_name: String::new(),
		family: 0,
		model: 0,
		stepping: 0,
		features: Vec::new(),
		cache_size: 0,
		core_count: 1,
		thread_count: 1,
	};

	// Get CPU vendor
	let (_, vendor_ebx, vendor_ecx, vendor_edx) = cpuid(0);
	cpu_info.vendor = format!(
		"{}{}{}",
		u32_to_string(vendor_ebx),
		u32_to_string(vendor_edx),
		u32_to_string(vendor_ecx)
	);

	// Get CPU features and family/model
	let (version_eax, _ebx, feature_ecx, feature_edx) = cpuid(1);
	cpu_info.family = (version_eax >> 8) & 0xF;
	cpu_info.model = (version_eax >> 4) & 0xF;
	cpu_info.stepping = version_eax & 0xF;

	// Extended family/model for newer CPUs
	if cpu_info.family == 0xF {
		cpu_info.family += (version_eax >> 20) & 0xFF;
	}
	if cpu_info.family == 0x6 || cpu_info.family == 0xF {
		cpu_info.model += ((version_eax >> 16) & 0xF) << 4;
	}

	// Check for common features
	if feature_edx & (1 << 23) != 0 {
		cpu_info.features.push("MMX".to_string());
	}
	if feature_edx & (1 << 25) != 0 {
		cpu_info.features.push("SSE".to_string());
	}
	if feature_edx & (1 << 26) != 0 {
		cpu_info.features.push("SSE2".to_string());
	}
	if feature_ecx & (1 << 0) != 0 {
		cpu_info.features.push("SSE3".to_string());
	}
	if feature_ecx & (1 << 9) != 0 {
		cpu_info.features.push("SSSE3".to_string());
	}
	if feature_ecx & (1 << 19) != 0 {
		cpu_info.features.push("SSE4.1".to_string());
	}
	if feature_ecx & (1 << 20) != 0 {
		cpu_info.features.push("SSE4.2".to_string());
	}

	// Get model name from extended CPUID
	let max_extended = cpuid(0x80000000).0;
	if max_extended >= 0x80000004 {
		let mut model_name = String::new();
		for i in 0x80000002..=0x80000004 {
			let (eax, ebx, ecx, edx) = cpuid(i);
			model_name.push_str(&u32_to_string(eax));
			model_name.push_str(&u32_to_string(ebx));
			model_name.push_str(&u32_to_string(ecx));
			model_name.push_str(&u32_to_string(edx));
		}
		cpu_info.model_name = model_name.trim().to_string();
	}

	Ok(cpu_info)
}

/// Detect system memory
pub fn detect_memory() -> Result<usize> {
	// Use multiple methods to detect memory

	// Method 1: CMOS
	let cmos_memory = unsafe {
		crate::arch::x86_64::port::outb(0x70, 0x17);
		let low = crate::arch::x86_64::port::inb(0x71) as usize;
		crate::arch::x86_64::port::outb(0x70, 0x18);
		let high = crate::arch::x86_64::port::inb(0x71) as usize;

		let extended_mem = (high << 8) | low; // in KB
		1024 * 1024 + (extended_mem * 1024) // Base 1MB + extended
	};

	// Method 2: Try to probe memory (simplified)
	let probe_memory = probe_memory_size();

	// Use the larger of the two methods
	let detected_memory = core::cmp::max(cmos_memory, probe_memory);

	// Sanity check
	if detected_memory < 16 * 1024 * 1024 {
		Ok(64 * 1024 * 1024) // Default to 64MB
	} else if detected_memory > 16 * 1024 * 1024 * 1024 {
		Ok(8 * 1024 * 1024 * 1024) // Cap at 8GB
	} else {
		Ok(detected_memory)
	}
}

/// Probe memory size by testing access
fn probe_memory_size() -> usize {
	// Simplified memory probing - just return a reasonable default
	// In a real implementation, this would carefully probe memory ranges
	512 * 1024 * 1024 // 512MB default
}

/// Detect PCI devices
pub fn detect_pci_devices() -> Result<Vec<PciDevice>> {
	let mut devices = Vec::new();

	// Scan PCI bus 0 (simplified)
	for device in 0..32 {
		for function in 0..8 {
			let vendor_id = pci_config_read(0, device, function, 0x00) as u16;
			if vendor_id != 0xFFFF {
				let device_id =
					(pci_config_read(0, device, function, 0x00) >> 16) as u16;
				let class_info = pci_config_read(0, device, function, 0x08);
				let revision =
					(pci_config_read(0, device, function, 0x08) & 0xFF) as u8;
				let mut bars = [PciBar::new(); 6];
				for i in 0..6 {
					let bar_val = pci_config_read(
						0,
						device,
						function,
						0x10 + (i * 4),
					);
					if bar_val == 0 {
						continue;
					}
					let is_io = bar_val & 1 != 0;
					if is_io {
						bars[i as usize].address =
							(bar_val & 0xFFFFFFFC) as u64;
					} else {
						bars[i as usize].address =
							(bar_val & 0xFFFFFFF0) as u64;
					}
					bars[i as usize].flags = bar_val & 0xF;
				}

				devices.push(PciDevice {
					bus: 0,
					slot: device,
					function,
					vendor: vendor_id,
					device: device_id,
					class: (class_info >> 16),
					revision,
					subsystem_vendor: 0, // Not implemented
					subsystem_device: 0, // Not implemented
					irq: 0,              // Not implemented
					bars,
				});
			}
		}
	}

	Ok(devices)
}

/// Read PCI configuration space
pub(crate) fn pci_config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
	let address = 0x80000000u32
		| ((bus as u32) << 16)
		| ((device as u32) << 11)
		| ((function as u32) << 8)
		| (offset as u32 & 0xFC);

	unsafe {
		crate::arch::x86_64::port::outl(0xCF8, address);
		crate::arch::x86_64::port::inl(0xCFC)
	}
}

/// Execute CPUID instruction (simplified to avoid RBX conflicts)
fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
	// For now, return simplified values to avoid RBX register conflicts
	match leaf {
		0 => (0x0000000D, 0x756e6547, 0x6c65746e, 0x49656e69), // "GenuineIntel"
		1 => (0x000906E9, 0x00100800, 0x7FFAFBBF, 0xBFEBFBFF), // Typical Intel CPU
		0x80000000 => (0x80000008, 0, 0, 0),
		0x80000002 => (0x65746E49, 0x2952286C, 0x726F4320, 0x4D542865), /* "Intel(R) Core(TM" */
		0x80000003 => (0x35692029, 0x3034332D, 0x20555043, 0x20402030), // ") i5-4340
		// CPU @ "
		0x80000004 => (0x30302E33, 0x007A4847, 0x00000000, 0x00000000), // "3.00GHz"
		_ => (0, 0, 0, 0),
	}
}

/// Convert u32 to 4-character string
fn u32_to_string(value: u32) -> String {
	let bytes = value.to_le_bytes();
	String::from_utf8_lossy(&bytes)
		.trim_end_matches('\0')
		.to_string()
}

/// Get system information
pub fn get_system_info() -> Result<SystemInfo> {
	let cpu = detect_cpu()?;
	let total_memory = detect_memory()?;
	let pci_devices = detect_pci_devices()?;

	Ok(SystemInfo {
		cpu,
		total_memory,
		available_memory: (total_memory * 95) / 100,
		boot_device: "Unknown".to_string(),
		acpi_available: false, // TODO: Implement ACPI detection
		pci_devices,
	})
}
