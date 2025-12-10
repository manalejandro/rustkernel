// SPDX-License-Identifier: GPL-2.0

//! Kernel logging and debugging system

use alloc::{format, string::String, vec, vec::Vec};
use core::fmt::Write;

use crate::error::Result;
use crate::sync::Spinlock;
use crate::time::get_jiffies;

/// Log levels (compatible with Linux kernel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
	Emergency = 0, // KERN_EMERG
	Alert = 1,     // KERN_ALERT
	Critical = 2,  // KERN_CRIT
	Error = 3,     // KERN_ERR
	Warning = 4,   // KERN_WARNING
	Notice = 5,    // KERN_NOTICE
	Info = 6,      // KERN_INFO
	Debug = 7,     // KERN_DEBUG
}

impl LogLevel {
	pub fn as_str(&self) -> &'static str {
		match self {
			LogLevel::Emergency => "EMERG",
			LogLevel::Alert => "ALERT",
			LogLevel::Critical => "CRIT",
			LogLevel::Error => "ERROR",
			LogLevel::Warning => "WARN",
			LogLevel::Notice => "NOTICE",
			LogLevel::Info => "INFO",
			LogLevel::Debug => "DEBUG",
		}
	}

	pub fn color_code(&self) -> &'static str {
		match self {
			LogLevel::Emergency => "\x1b[95m", // Magenta
			LogLevel::Alert => "\x1b[91m",     // Bright Red
			LogLevel::Critical => "\x1b[31m",  // Red
			LogLevel::Error => "\x1b[31m",     // Red
			LogLevel::Warning => "\x1b[33m",   // Yellow
			LogLevel::Notice => "\x1b[36m",    // Cyan
			LogLevel::Info => "\x1b[32m",      // Green
			LogLevel::Debug => "\x1b[37m",     // White
		}
	}
}

/// Log entry structure
#[derive(Debug, Clone)]
pub struct LogEntry {
	pub level: LogLevel,
	pub timestamp: u64,
	pub cpu: u32,
	pub pid: Option<u32>,
	pub module: String,
	pub message: String,
}

impl LogEntry {
	pub fn new(level: LogLevel, module: String, message: String) -> Self {
		Self {
			level,
			timestamp: get_jiffies().0,
			cpu: 0, // TODO: Get current CPU ID
			pid: crate::process::current_process_pid().map(|p| p.0),
			module,
			message,
		}
	}

	pub fn format(&self, colored: bool) -> String {
		let color_start = if colored { self.level.color_code() } else { "" };
		let color_reset = if colored { "\x1b[0m" } else { "" };

		format!(
			"{}[{:>5}] [{:>10}] {} {}: {}{}\n",
			color_start,
			self.level.as_str(),
			self.timestamp,
			self.cpu,
			self.module,
			self.message,
			color_reset
		)
	}
}

/// Debug categories for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCategory {
	Memory,
	Process,
	FileSystem,
	Network,
	Driver,
	Interrupt,
	Scheduler,
	UserMode,
	Performance,
	All,
}

/// Logger configuration
#[derive(Debug)]
pub struct LoggerConfig {
	pub min_level: LogLevel,
	pub max_entries: usize,
	pub console_output: bool,
	pub colored_output: bool,
	pub debug_categories: Vec<DebugCategory>,
}

impl LoggerConfig {
	pub fn new() -> Self {
		Self {
			min_level: LogLevel::Info,
			max_entries: 1000,
			console_output: true,
			colored_output: true,
			debug_categories: vec![DebugCategory::All],
		}
	}

	pub fn with_level(mut self, level: LogLevel) -> Self {
		self.min_level = level;
		self
	}

	pub fn with_max_entries(mut self, max: usize) -> Self {
		self.max_entries = max;
		self
	}

	pub fn enable_category(mut self, category: DebugCategory) -> Self {
		if !self.debug_categories.contains(&category) {
			self.debug_categories.push(category);
		}
		self
	}
}

/// Kernel logger
pub struct KernelLogger {
	config: LoggerConfig,
	entries: Vec<LogEntry>,
	stats: LogStats,
}

/// Logging statistics
#[derive(Debug, Default)]
pub struct LogStats {
	pub total_entries: u64,
	pub entries_by_level: [u64; 8], // One for each log level
	pub dropped_entries: u64,
}

impl KernelLogger {
	pub const fn new() -> Self {
		Self {
			config: LoggerConfig {
				min_level: LogLevel::Info,
				max_entries: 1000,
				console_output: true,
				colored_output: true,
				debug_categories: Vec::new(),
			},
			entries: Vec::new(),
			stats: LogStats {
				total_entries: 0,
				entries_by_level: [0; 8],
				dropped_entries: 0,
			},
		}
	}

	pub fn init(&mut self, config: LoggerConfig) {
		self.config = config;
	}

	pub fn log(&mut self, level: LogLevel, module: &str, message: &str) {
		// Check if we should log this level
		if level > self.config.min_level {
			return;
		}

		let entry = LogEntry::new(level, module.into(), message.into());

		// Update statistics
		self.stats.total_entries += 1;
		self.stats.entries_by_level[level as usize] += 1;

		// Output to console if enabled
		if self.config.console_output {
			let formatted = entry.format(self.config.colored_output);
			// Use the print macro since there's no direct write_str function
			crate::print!("{}", formatted);
		}

		// Store in buffer
		if self.entries.len() >= self.config.max_entries {
			self.entries.remove(0); // Remove oldest entry
			self.stats.dropped_entries += 1;
		}
		self.entries.push(entry);
	}

	pub fn get_entries(&self) -> &[LogEntry] {
		&self.entries
	}

	pub fn get_entries_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
		self.entries.iter().filter(|e| e.level == level).collect()
	}

	pub fn clear(&mut self) {
		self.entries.clear();
	}

	pub fn get_stats(&self) -> &LogStats {
		&self.stats
	}

	pub fn set_level(&mut self, level: LogLevel) {
		self.config.min_level = level;
	}

	pub fn dump_buffer(&self) -> String {
		let mut output = String::new();
		for entry in &self.entries {
			output.push_str(&entry.format(false));
		}
		output
	}

	pub fn generate_report(&self) -> String {
		let mut report = String::from("Kernel Logger Report\n");
		report.push_str("====================\n\n");

		report.push_str(&format!("Configuration:\n"));
		report.push_str(&format!("  Min Level: {:?}\n", self.config.min_level));
		report.push_str(&format!("  Max Entries: {}\n", self.config.max_entries));
		report.push_str(&format!(
			"  Console Output: {}\n",
			self.config.console_output
		));
		report.push_str(&format!(
			"  Colored Output: {}\n",
			self.config.colored_output
		));

		report.push_str(&format!("\nStatistics:\n"));
		report.push_str(&format!("  Total Entries: {}\n", self.stats.total_entries));
		report.push_str(&format!(
			"  Dropped Entries: {}\n",
			self.stats.dropped_entries
		));
		report.push_str(&format!("  Current Buffer Size: {}\n", self.entries.len()));

		report.push_str(&format!("\nEntries by Level:\n"));
		for (i, &count) in self.stats.entries_by_level.iter().enumerate() {
			if count > 0 {
				let level = match i {
					0 => LogLevel::Emergency,
					1 => LogLevel::Alert,
					2 => LogLevel::Critical,
					3 => LogLevel::Error,
					4 => LogLevel::Warning,
					5 => LogLevel::Notice,
					6 => LogLevel::Info,
					7 => LogLevel::Debug,
					_ => continue,
				};
				report.push_str(&format!("  {:?}: {}\n", level, count));
			}
		}

		if !self.entries.is_empty() {
			report.push_str(&format!(
				"\nRecent Entries ({}):\n",
				core::cmp::min(10, self.entries.len())
			));
			for entry in self.entries.iter().rev().take(10) {
				report.push_str(&format!("  {}\n", entry.format(false).trim()));
			}
		}

		report
	}
}

/// Global kernel logger
static KERNEL_LOGGER: Spinlock<Option<KernelLogger>> = Spinlock::new(None);

/// Initialize kernel logging system
pub fn init_logging() -> Result<()> {
	let mut logger = KERNEL_LOGGER.lock();
	*logger = Some(KernelLogger::new());

	if let Some(ref mut l) = *logger {
		let config = LoggerConfig::new()
			.with_level(LogLevel::Info)
			.with_max_entries(2000);
		l.init(config);
	}

	// Log initialization message
	log_info("logging", "Kernel logging system initialized");
	Ok(())
}

/// Main logging function
pub fn log(level: LogLevel, module: &str, message: &str) {
	let mut logger = KERNEL_LOGGER.lock();
	if let Some(ref mut l) = *logger {
		l.log(level, module, message);
	}
}

/// Convenience logging functions
pub fn log_emergency(module: &str, message: &str) {
	log(LogLevel::Emergency, module, message);
}

pub fn log_alert(module: &str, message: &str) {
	log(LogLevel::Alert, module, message);
}

pub fn log_critical(module: &str, message: &str) {
	log(LogLevel::Critical, module, message);
}

pub fn log_error(module: &str, message: &str) {
	log(LogLevel::Error, module, message);
}

pub fn log_warning(module: &str, message: &str) {
	log(LogLevel::Warning, module, message);
}

pub fn log_notice(module: &str, message: &str) {
	log(LogLevel::Notice, module, message);
}

pub fn log_info(module: &str, message: &str) {
	log(LogLevel::Info, module, message);
}

pub fn log_debug(module: &str, message: &str) {
	log(LogLevel::Debug, module, message);
}

/// Get logging statistics
pub fn get_log_stats() -> Option<LogStats> {
	let logger = KERNEL_LOGGER.lock();
	if let Some(ref l) = *logger {
		Some(LogStats {
			total_entries: l.stats.total_entries,
			entries_by_level: l.stats.entries_by_level,
			dropped_entries: l.stats.dropped_entries,
		})
	} else {
		None
	}
}

/// Generate logging report
pub fn generate_log_report() -> String {
	let logger = KERNEL_LOGGER.lock();
	if let Some(ref l) = *logger {
		l.generate_report()
	} else {
		"Logging system not initialized".into()
	}
}

/// Dump log buffer
pub fn dump_log_buffer() -> String {
	let logger = KERNEL_LOGGER.lock();
	if let Some(ref l) = *logger {
		l.dump_buffer()
	} else {
		"Logging system not initialized".into()
	}
}

/// Clear log buffer
pub fn clear_log_buffer() {
	let mut logger = KERNEL_LOGGER.lock();
	if let Some(ref mut l) = *logger {
		l.clear();
	}
}

/// Set log level
pub fn set_log_level(level: LogLevel) {
	let mut logger = KERNEL_LOGGER.lock();
	if let Some(ref mut l) = *logger {
		l.set_level(level);
	}
}

/// Debugging macros
#[macro_export]
macro_rules! debug_print {
    ($category:expr, $($arg:tt)*) => {
        crate::logging::log_debug(stringify!($category), &alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! trace_function {
	($func:expr) => {
		crate::logging::log_debug("trace", &alloc::format!("Entering function: {}", $func));
	};
}

/// Kernel assertions with logging
#[macro_export]
macro_rules! kernel_assert {
	($cond:expr) => {
		if !$cond {
			crate::logging::log_critical(
				"assert",
				&alloc::format!(
					"Assertion failed: {} at {}:{}",
					stringify!($cond),
					file!(),
					line!()
				),
			);
			panic!("Kernel assertion failed: {}", stringify!($cond));
		}
	};
	($cond:expr, $msg:expr) => {
		if !$cond {
			crate::logging::log_critical(
				"assert",
				&alloc::format!(
					"Assertion failed: {} - {} at {}:{}",
					stringify!($cond),
					$msg,
					file!(),
					line!()
				),
			);
			panic!("Kernel assertion failed: {} - {}", stringify!($cond), $msg);
		}
	};
}

/// Memory debugging helpers
pub mod debug {
	use super::*;

	pub fn dump_memory(addr: usize, size: usize, label: &str) {
		let mut output = format!(
			"Memory dump: {} (addr: 0x{:x}, size: {})\n",
			label, addr, size
		);

		unsafe {
			let ptr = addr as *const u8;
			for i in 0..core::cmp::min(size, 256) {
				if i % 16 == 0 {
					output.push_str(&format!("\n{:08x}: ", addr + i));
				}
				output.push_str(&format!("{:02x} ", *ptr.add(i)));
			}
		}

		log_debug("memory", &output);
	}

	pub fn log_stack_trace() {
		// TODO: Implement proper stack unwinding
		log_debug("stack", "Stack trace not yet implemented");
	}

	pub fn check_kernel_stack() {
		// TODO: Check if kernel stack is getting low
		log_debug("stack", "Kernel stack check not yet implemented");
	}
}
