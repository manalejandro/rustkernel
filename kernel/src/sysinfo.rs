// SPDX-License-Identifier: GPL-2.0

//! System information and hardware detection

use alloc::{format, string::String, vec::Vec};

use crate::error::Result;
use crate::sync::Spinlock;

/// CPU information structure
#[derive(Debug, Clone)]
pub struct CpuInfo {
	pub vendor: String,
	pub model_name: String,
	pub family: u32,
	pub model: u32,
	pub stepping: u32,
	pub features: Vec<String>,
	pub cache_size: Option<usize>,
	pub frequency: Option<u64>, // MHz
	pub cores: u32,
	pub threads: u32,
}

impl CpuInfo {
	pub fn new() -> Self {
		Self {
			vendor: "Unknown".into(),
			model_name: "Unknown CPU".into(),
			family: 0,
			model: 0,
			stepping: 0,
			features: Vec::new(),
			cache_size: None,
			frequency: None,
			cores: 1,
			threads: 1,
		}
	}

	pub fn detect() -> Self {
		let mut info = Self::new();

		// Basic CPUID detection for x86_64
		#[cfg(target_arch = "x86_64")]
		{
			info.detect_x86_64();
		}

		info
	}

	#[cfg(target_arch = "x86_64")]
	fn detect_x86_64(&mut self) {
		use core::arch::asm;

		// Check if CPUID is supported
		let mut eax: u32;
		let mut ebx: u32 = 0;
		let mut ecx: u32 = 0;
		let mut edx: u32 = 0;

		unsafe {
			asm!("cpuid", inout("eax") 0 => eax, out("ecx") _, out("edx") _, options(preserves_flags));
		}

		if eax >= 1 {
			// Get basic CPU info - avoid using ebx directly due to LLVM restrictions
			unsafe {
				asm!("mov {ebx_save}, rbx",
                     "cpuid", 
                     "mov {ebx_out:e}, ebx",
                     "mov rbx, {ebx_save}",
                     ebx_save = out(reg) _,
                     ebx_out = out(reg) ebx,
                     inout("eax") 1 => eax, 
                     out("ecx") ecx, 
                     out("edx") edx,
                     options(preserves_flags));
			}

			self.family = ((eax >> 8) & 0xF) as u32;
			self.model = ((eax >> 4) & 0xF) as u32;
			self.stepping = (eax & 0xF) as u32;

			// Detect features
			if edx & (1 << 0) != 0 {
				self.features.push("FPU".into());
			}
			if edx & (1 << 4) != 0 {
				self.features.push("TSC".into());
			}
			if edx & (1 << 5) != 0 {
				self.features.push("MSR".into());
			}
			if edx & (1 << 6) != 0 {
				self.features.push("PAE".into());
			}
			if edx & (1 << 8) != 0 {
				self.features.push("CX8".into());
			}
			if edx & (1 << 11) != 0 {
				self.features.push("SEP".into());
			}
			if edx & (1 << 13) != 0 {
				self.features.push("PGE".into());
			}
			if edx & (1 << 15) != 0 {
				self.features.push("CMOV".into());
			}
			if edx & (1 << 23) != 0 {
				self.features.push("MMX".into());
			}
			if edx & (1 << 25) != 0 {
				self.features.push("SSE".into());
			}
			if edx & (1 << 26) != 0 {
				self.features.push("SSE2".into());
			}

			if ecx & (1 << 0) != 0 {
				self.features.push("SSE3".into());
			}
			if ecx & (1 << 9) != 0 {
				self.features.push("SSSE3".into());
			}
			if ecx & (1 << 19) != 0 {
				self.features.push("SSE4.1".into());
			}
			if ecx & (1 << 20) != 0 {
				self.features.push("SSE4.2".into());
			}
			if ecx & (1 << 28) != 0 {
				self.features.push("AVX".into());
			}
		}

		// Try to get vendor string
		unsafe {
			let mut vendor_eax: u32;
			let mut vendor_ebx: u32;
			let mut vendor_ecx: u32;
			let mut vendor_edx: u32;

			asm!("mov {ebx_save}, rbx",
                 "cpuid", 
                 "mov {ebx_out:e}, ebx",
                 "mov rbx, {ebx_save}",
                 ebx_save = out(reg) _,
                 ebx_out = out(reg) vendor_ebx,
                 inout("eax") 0 => vendor_eax, 
                 out("ecx") vendor_ecx, 
                 out("edx") vendor_edx,
                 options(preserves_flags));

			if vendor_eax >= 0 {
				let mut vendor_string = [0u8; 12];
				vendor_string[0..4].copy_from_slice(&vendor_ebx.to_le_bytes());
				vendor_string[4..8].copy_from_slice(&vendor_edx.to_le_bytes());
				vendor_string[8..12].copy_from_slice(&vendor_ecx.to_le_bytes());

				if let Ok(vendor) = core::str::from_utf8(&vendor_string) {
					self.vendor = vendor.into();
				}
			}
		}
	}
}

/// Memory information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
	pub total_ram: usize,
	pub available_ram: usize,
	pub used_ram: usize,
	pub kernel_memory: usize,
	pub user_memory: usize,
	pub cache_memory: usize,
	pub swap_total: usize,
	pub swap_used: usize,
}

impl MemoryInfo {
	pub fn detect() -> Self {
		let boot_info = unsafe { crate::boot::get_boot_info() };
		let (total_pages, allocated_pages, free_pages) = crate::memory::page::stats();

		let page_size = 4096; // 4KB pages
		let total_ram = total_pages * page_size;
		let used_ram = allocated_pages * page_size;
		let available_ram = free_pages * page_size;

		Self {
			total_ram,
			available_ram,
			used_ram,
			kernel_memory: used_ram, // Simplified for now
			user_memory: 0,
			cache_memory: 0,
			swap_total: 0,
			swap_used: 0,
		}
	}
}

/// System uptime and load information
#[derive(Debug, Clone)]
pub struct SystemStats {
	pub uptime_seconds: u64,
	pub boot_time: u64,
	pub processes: u32,
	pub threads: u32,
	pub load_average: (f32, f32, f32), // 1min, 5min, 15min
	pub context_switches: u64,
	pub interrupts: u64,
}

impl SystemStats {
	pub fn collect() -> Self {
		let uptime = crate::time::get_jiffies().as_u64() / 1000; // Convert to seconds

		// Collect performance counters
		let context_switches =
			crate::perf::perf_counter_get(crate::perf::CounterType::ContextSwitches)
				.unwrap_or(0);
		let interrupts =
			crate::perf::perf_counter_get(crate::perf::CounterType::Interrupts)
				.unwrap_or(0);

		Self {
			uptime_seconds: uptime,
			boot_time: 0,                  // TODO: Get actual boot time
			processes: 1,                  // TODO: Count actual processes
			threads: 1,                    // TODO: Count actual threads
			load_average: (0.0, 0.0, 0.0), // TODO: Calculate load average
			context_switches,
			interrupts,
		}
	}
}

/// Hardware device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
	pub name: String,
	pub device_type: String,
	pub vendor: Option<String>,
	pub device_id: Option<u32>,
	pub driver: Option<String>,
	pub status: String,
}

/// Complete system information
#[derive(Debug)]
pub struct SystemInfo {
	pub kernel_version: String,
	pub architecture: String,
	pub cpu_info: CpuInfo,
	pub memory_info: MemoryInfo,
	pub system_stats: SystemStats,
	pub devices: Vec<DeviceInfo>,
}

impl SystemInfo {
	pub fn collect() -> Self {
		Self {
			kernel_version: format!("{} v{}", crate::NAME, crate::VERSION),
			architecture: "x86_64".into(),
			cpu_info: CpuInfo::detect(),
			memory_info: MemoryInfo::detect(),
			system_stats: SystemStats::collect(),
			devices: Vec::new(), // TODO: Enumerate devices
		}
	}

	pub fn format_detailed(&self) -> String {
		let mut output = String::new();

		output.push_str("System Information\n");
		output.push_str("==================\n\n");

		output.push_str(&format!("Kernel: {}\n", self.kernel_version));
		output.push_str(&format!("Architecture: {}\n", self.architecture));
		output.push_str(&format!(
			"Uptime: {} seconds\n",
			self.system_stats.uptime_seconds
		));

		output.push_str("\nCPU Information:\n");
		output.push_str(&format!("  Vendor: {}\n", self.cpu_info.vendor));
		output.push_str(&format!("  Model: {}\n", self.cpu_info.model_name));
		output.push_str(&format!(
			"  Family: {}, Model: {}, Stepping: {}\n",
			self.cpu_info.family, self.cpu_info.model, self.cpu_info.stepping
		));
		output.push_str(&format!(
			"  Cores: {}, Threads: {}\n",
			self.cpu_info.cores, self.cpu_info.threads
		));
		if !self.cpu_info.features.is_empty() {
			output.push_str(&format!(
				"  Features: {}\n",
				self.cpu_info.features.join(", ")
			));
		}

		output.push_str("\nMemory Information:\n");
		output.push_str(&format!(
			"  Total RAM: {} KB\n",
			self.memory_info.total_ram / 1024
		));
		output.push_str(&format!(
			"  Available RAM: {} KB\n",
			self.memory_info.available_ram / 1024
		));
		output.push_str(&format!(
			"  Used RAM: {} KB\n",
			self.memory_info.used_ram / 1024
		));
		output.push_str(&format!(
			"  Kernel Memory: {} KB\n",
			self.memory_info.kernel_memory / 1024
		));

		output.push_str("\nSystem Statistics:\n");
		output.push_str(&format!("  Processes: {}\n", self.system_stats.processes));
		output.push_str(&format!("  Threads: {}\n", self.system_stats.threads));
		output.push_str(&format!(
			"  Context Switches: {}\n",
			self.system_stats.context_switches
		));
		output.push_str(&format!("  Interrupts: {}\n", self.system_stats.interrupts));

		if !self.devices.is_empty() {
			output.push_str("\nDevices:\n");
			for device in &self.devices {
				output.push_str(&format!(
					"  {} ({}): {}\n",
					device.name, device.device_type, device.status
				));
			}
		}

		output
	}

	pub fn format_compact(&self) -> String {
		format!(
			"{} {} - Uptime: {}s, RAM: {}/{} KB, CPU: {}",
			self.kernel_version,
			self.architecture,
			self.system_stats.uptime_seconds,
			self.memory_info.used_ram / 1024,
			self.memory_info.total_ram / 1024,
			self.cpu_info.vendor
		)
	}
}

/// Global system information cache
static SYSTEM_INFO_CACHE: Spinlock<Option<SystemInfo>> = Spinlock::new(None);

/// Initialize system information collection
pub fn init_sysinfo() -> Result<()> {
	let mut cache = SYSTEM_INFO_CACHE.lock();
	*cache = Some(SystemInfo::collect());

	crate::info!("System information collection initialized");
	Ok(())
}

/// Get current system information (cached)
pub fn get_system_info() -> SystemInfo {
	let mut cache = SYSTEM_INFO_CACHE.lock();

	// Refresh cache with current data
	*cache = Some(SystemInfo::collect());

	if let Some(ref info) = *cache {
		// Create a copy since we can't return a reference
		SystemInfo {
			kernel_version: info.kernel_version.clone(),
			architecture: info.architecture.clone(),
			cpu_info: info.cpu_info.clone(),
			memory_info: info.memory_info.clone(),
			system_stats: info.system_stats.clone(),
			devices: info.devices.clone(),
		}
	} else {
		SystemInfo::collect()
	}
}

/// Get formatted system information
pub fn get_system_info_detailed() -> String {
	let info = get_system_info();
	info.format_detailed()
}

/// Get compact system information
pub fn get_system_info_compact() -> String {
	let info = get_system_info();
	info.format_compact()
}

/// CPU benchmark utilities
pub mod benchmark {
	use super::*;

	pub fn cpu_speed_test() -> u64 {
		let start = crate::time::get_jiffies();

		// Simple CPU-intensive operation
		let mut result = 0u64;
		for i in 0..1000000 {
			result = result.wrapping_add(i * i);
		}

		let end = crate::time::get_jiffies();
		let duration = (end - start).as_u64();

		// Prevent optimization from removing the loop
		core::hint::black_box(result);

		duration
	}

	pub fn memory_speed_test() -> u64 {
		// TODO: Implement memory speed test
		0
	}
}
