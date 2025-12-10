// SPDX-License-Identifier: GPL-2.0

//! Kernel diagnostics and health monitoring

use alloc::{format, string::String, vec::Vec};
use core::fmt::Write;

use crate::error::Result;
use crate::sync::Spinlock;
use crate::time::get_jiffies;
use crate::types::Jiffies;

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
	Healthy,
	Warning,
	Critical,
	Unknown,
}

/// Diagnostic category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCategory {
	Memory,
	CPU,
	IO,
	Network,
	FileSystem,
	Process,
	Kernel,
}

/// Diagnostic entry
#[derive(Debug, Clone)]
pub struct DiagnosticEntry {
	pub category: DiagnosticCategory,
	pub status: HealthStatus,
	pub message: String,
	pub timestamp: Jiffies,
	pub details: Option<String>,
}

/// System diagnostics
pub struct SystemDiagnostics {
	entries: Vec<DiagnosticEntry>,
	last_check: Jiffies,
	health_status: HealthStatus,
}

static DIAGNOSTICS: Spinlock<SystemDiagnostics> = Spinlock::new(SystemDiagnostics::new());

impl SystemDiagnostics {
	const fn new() -> Self {
		Self {
			entries: Vec::new(),
			last_check: Jiffies(0),
			health_status: HealthStatus::Unknown,
		}
	}

	fn add_entry(&mut self, entry: DiagnosticEntry) {
		// Keep only the last 1000 entries
		if self.entries.len() >= 1000 {
			self.entries.remove(0);
		}

		// Update overall health status
		match entry.status {
			HealthStatus::Critical => self.health_status = HealthStatus::Critical,
			HealthStatus::Warning if self.health_status != HealthStatus::Critical => {
				self.health_status = HealthStatus::Warning;
			}
			HealthStatus::Healthy if self.health_status == HealthStatus::Unknown => {
				self.health_status = HealthStatus::Healthy;
			}
			_ => {}
		}

		self.entries.push(entry);
	}

	fn get_entries_by_category(&self, category: DiagnosticCategory) -> Vec<&DiagnosticEntry> {
		self.entries
			.iter()
			.filter(|entry| entry.category == category)
			.collect()
	}

	fn get_recent_entries(&self, max_age_jiffies: u64) -> Vec<&DiagnosticEntry> {
		let current_time = get_jiffies();
		self.entries
			.iter()
			.filter(|entry| {
				(current_time - entry.timestamp).as_u64() <= max_age_jiffies
			})
			.collect()
	}
}

/// Initialize diagnostics system
pub fn init_diagnostics() -> Result<()> {
	let mut diag = DIAGNOSTICS.lock();
	diag.last_check = get_jiffies();
	diag.health_status = HealthStatus::Healthy;

	// Add initial diagnostic entry
	let entry = DiagnosticEntry {
		category: DiagnosticCategory::Kernel,
		status: HealthStatus::Healthy,
		message: "Diagnostics system initialized".into(),
		timestamp: get_jiffies(),
		details: None,
	};
	diag.add_entry(entry);

	Ok(())
}

/// Add a diagnostic entry
pub fn add_diagnostic(
	category: DiagnosticCategory,
	status: HealthStatus,
	message: &str,
	details: Option<&str>,
) {
	let mut diag = DIAGNOSTICS.lock();
	let entry = DiagnosticEntry {
		category,
		status,
		message: message.into(),
		timestamp: get_jiffies(),
		details: details.map(|s| s.into()),
	};
	diag.add_entry(entry);
}

/// Get system health status
pub fn get_health_status() -> HealthStatus {
	let diag = DIAGNOSTICS.lock();
	diag.health_status
}

/// Run system health check
pub fn run_health_check() -> Result<()> {
	let mut issues_found = 0;

	// Check memory usage
	if let Ok(stats) = crate::memory::get_memory_stats() {
		if stats.usage_percent > 90 {
			add_diagnostic(
				DiagnosticCategory::Memory,
				HealthStatus::Critical,
				"High memory usage",
				Some(&format!("Memory usage: {}%", stats.usage_percent)),
			);
			issues_found += 1;
		} else if stats.usage_percent > 75 {
			add_diagnostic(
				DiagnosticCategory::Memory,
				HealthStatus::Warning,
				"Elevated memory usage",
				Some(&format!("Memory usage: {}%", stats.usage_percent)),
			);
		}
	}

	// Check kernel threads
	if let Ok(thread_count) = crate::kthread::get_thread_count() {
		if thread_count == 0 {
			add_diagnostic(
				DiagnosticCategory::Kernel,
				HealthStatus::Warning,
				"No kernel threads running",
				None,
			);
		}
	}

	// Check file system
	if let Ok(fs_stats) = crate::memfs::get_filesystem_stats() {
		if fs_stats.files_count > 10000 {
			add_diagnostic(
				DiagnosticCategory::FileSystem,
				HealthStatus::Warning,
				"High number of files",
				Some(&format!("Files: {}", fs_stats.files_count)),
			);
		}
	}

	// Update last check time
	{
		let mut diag = DIAGNOSTICS.lock();
		diag.last_check = get_jiffies();
	}

	if issues_found == 0 {
		add_diagnostic(
			DiagnosticCategory::Kernel,
			HealthStatus::Healthy,
			"Health check completed - system healthy",
			None,
		);
	}

	Ok(())
}

/// Get diagnostic report
pub fn get_diagnostic_report() -> String {
	let diag = DIAGNOSTICS.lock();
	let mut report = String::new();

	writeln!(&mut report, "=== System Diagnostics Report ===").unwrap();
	writeln!(&mut report, "Overall Health: {:?}", diag.health_status).unwrap();
	writeln!(&mut report, "Last Check: {}", diag.last_check.as_u64()).unwrap();
	writeln!(&mut report, "Total Entries: {}", diag.entries.len()).unwrap();
	writeln!(&mut report).unwrap();

	// Group by category
	for category in [
		DiagnosticCategory::Kernel,
		DiagnosticCategory::Memory,
		DiagnosticCategory::CPU,
		DiagnosticCategory::IO,
		DiagnosticCategory::Network,
		DiagnosticCategory::FileSystem,
		DiagnosticCategory::Process,
	] {
		let entries = diag.get_entries_by_category(category);
		if !entries.is_empty() {
			writeln!(&mut report, "{:?} ({} entries):", category, entries.len())
				.unwrap();
			for entry in entries.iter().rev().take(5) {
				// Show last 5 entries
				writeln!(
					&mut report,
					"  [{:?}] {} ({})",
					entry.status,
					entry.message,
					entry.timestamp.as_u64()
				)
				.unwrap();
				if let Some(details) = &entry.details {
					writeln!(&mut report, "    Details: {}", details).unwrap();
				}
			}
			writeln!(&mut report).unwrap();
		}
	}

	report
}

/// Get recent critical issues
pub fn get_critical_issues() -> Vec<DiagnosticEntry> {
	let diag = DIAGNOSTICS.lock();
	diag.entries
		.iter()
		.filter(|entry| entry.status == HealthStatus::Critical)
		.rev()
		.take(10)
		.cloned()
		.collect()
}

/// Clear diagnostic history
pub fn clear_diagnostics() {
	let mut diag = DIAGNOSTICS.lock();
	diag.entries.clear();
	diag.health_status = HealthStatus::Healthy;

	// Add cleared entry
	let entry = DiagnosticEntry {
		category: DiagnosticCategory::Kernel,
		status: HealthStatus::Healthy,
		message: "Diagnostic history cleared".into(),
		timestamp: get_jiffies(),
		details: None,
	};
	diag.add_entry(entry);
}

/// Automatic health monitoring task
pub fn health_monitor_task() {
	loop {
		// Run health check every 30 seconds (30000 jiffies at 1000 Hz)
		if let Err(_) = run_health_check() {
			add_diagnostic(
				DiagnosticCategory::Kernel,
				HealthStatus::Warning,
				"Health check failed",
				None,
			);
		}

		// Sleep for 30 seconds
		crate::kthread::sleep_for_jiffies(30000);
	}
}
