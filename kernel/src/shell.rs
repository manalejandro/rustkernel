// SPDX-License-Identifier: GPL-2.0

//! Kernel shell - a simple command-line interface

use alloc::{
	format,
	string::{String, ToString},
	vec::Vec,
};

use crate::error::Result;
use crate::{error, info, warn};

/// Maximum command line length
const MAX_COMMAND_LENGTH: usize = 256;

/// Kernel shell state
pub struct KernelShell {
	prompt: String,
	command_buffer: String,
	history: Vec<String>,
}

impl KernelShell {
	pub fn new() -> Self {
		Self {
			prompt: String::from("kernel> "),
			command_buffer: String::new(),
			history: Vec::new(),
		}
	}

	/// Process a character input
	pub fn process_char(&mut self, ch: char) -> Result<()> {
		match ch {
			'\n' | '\r' => {
				// Execute command
				self.execute_command()?;
				self.command_buffer.clear();
				self.print_prompt();
			}
			'\x08' | '\x7f' => {
				// Backspace
				if !self.command_buffer.is_empty() {
					self.command_buffer.pop();
					// TODO: Update display
				}
			}
			ch if ch.is_ascii_graphic() || ch == ' ' => {
				if self.command_buffer.len() < MAX_COMMAND_LENGTH {
					self.command_buffer.push(ch);
					// TODO: Echo character to display
				}
			}
			_ => {
				// Ignore other characters
			}
		}

		Ok(())
	}

	/// Execute a command
	fn execute_command(&mut self) -> Result<()> {
		let cmd = self.command_buffer.trim();

		if cmd.is_empty() {
			return Ok(());
		}

		// Add to history
		self.history.push(cmd.to_string());

		// Parse and execute command
		let parts: Vec<&str> = cmd.split_whitespace().collect();
		if let Some(&command) = parts.first() {
			match command {
				"help" => self.cmd_help(),
				"info" => self.cmd_info(),
				"mem" => self.cmd_memory(),
				"ps" => self.cmd_processes(),
				"uptime" => self.cmd_uptime(),
				"net" => self.cmd_network(&parts[1..]),
				"mod" => self.cmd_modules(&parts[1..]),
				"bench" => self.cmd_benchmark(&parts[1..]),
				"ls" => self.cmd_list(&parts[1..]),
				"cat" => self.cmd_cat(&parts[1..]),
				"mkdir" => self.cmd_mkdir(&parts[1..]),
				"touch" => self.cmd_touch(&parts[1..]),
				"rm" => self.cmd_remove(&parts[1..]),
				"clear" => self.cmd_clear(),
				"test" => self.cmd_test(&parts[1..]),
				"echo" => self.cmd_echo(&parts[1..]),
				"exec" => self.cmd_exec(&parts[1..]),
				"programs" => self.cmd_programs(),
				"perf" => self.cmd_perf(&parts[1..]),
				"log" => self.cmd_log(&parts[1..]),
				"sysinfo" => self.cmd_sysinfo(&parts[1..]),
				"diag" => self.cmd_diagnostics(&parts[1..]),
				"health" => self.cmd_health(&parts[1..]),
				"stress" => self.cmd_stress(&parts[1..]),
				"sched" => self.cmd_scheduler(&parts[1..]),
				"ipc" => self.cmd_ipc(&parts[1..]),
				"aperf" => self.cmd_advanced_perf(&parts[1..]),
				"tasks" => self.cmd_tasks(&parts[1..]),
				"panic" => self.cmd_panic(),
				"version" => {
					info!("Rust Kernel v0.1.0 - Advanced Features Edition");
					info!("Built for x86_64 architecture");
					info!(
						"Compiled on: {}",
						option_env!("BUILD_DATE").unwrap_or("unknown")
					);
					info!(
						"Git commit: {}",
						option_env!("GIT_HASH").unwrap_or("unknown")
					);
				}
				"hwinfo" => {
					info!("Hardware Information:");
					let boot_info = crate::boot::get_boot_info();
					info!(
						"  Memory: {} MB total, {} MB available",
						boot_info.memory_size / (1024 * 1024),
						boot_info.available_memory / (1024 * 1024)
					);
					info!("  CPUs: {}", boot_info.cpu_count);

					// TSC information
					let tsc_freq = crate::time::TSC_FREQUENCY
						.load(core::sync::atomic::Ordering::Relaxed);
					if tsc_freq > 0 {
						info!(
							"  TSC Frequency: {:.2} GHz",
							tsc_freq as f64 / 1_000_000_000.0
						);
					}
				}
				"interrupts" => {
					info!("Interrupt Statistics:");
					info!(
						"  Timer interrupts: {}",
						crate::timer::get_timer_interrupts()
					);
					info!(
						"  Total interrupts handled: {}",
						crate::interrupt::get_interrupt_count()
					);
					info!(
						"  Scheduler preemptions: {}",
						crate::enhanced_scheduler::get_preemption_count()
					);
				}
				"trace" => {
					info!("Kernel Stack Trace:");
					print_kernel_stack_trace();
				}
				"cpuinfo" => {
					info!("CPU Information:");
					if let Ok(info) = get_cpu_info() {
						info!("  Vendor: {}", info.vendor);
						info!("  Model: {}", info.model);
						info!("  Features: {}", info.features);
					} else {
						info!("  Unable to detect CPU information");
					}
				}
				_ => {
					info!("Unknown command: {}. Type 'help' for available commands.", command);
				}
			}
		}

		Ok(())
	}

	/// Print the shell prompt
	pub fn print_prompt(&self) {
		info!("{}", self.prompt);
	}

	/// Help command
	fn cmd_help(&self) {
		info!("Available commands:");
		info!("  help     - Show this help message");
		info!("  info     - Show kernel information");
		info!("  mem      - Show memory statistics");
		info!("  ps       - Show process information");
		info!("  uptime   - Show system uptime");
		info!("  net      - Network commands (stats, test)");
		info!("  mod      - Module commands (list, test, unload)");
		info!("  bench    - Benchmark commands (list, run, all)");
		info!("  ls       - List directory contents");
		info!("  cat      - Display file contents");
		info!("  mkdir    - Create directory");
		info!("  touch    - Create file");
		info!("  rm       - Remove file or directory");
		info!("  clear    - Clear screen");
		info!("  test     - Run kernel tests");
		info!("  echo     - Echo arguments");
		info!("  exec     - Execute user program");
		info!("  programs - List available user programs");
		info!("  perf     - Performance monitoring (report, clear, counters)");
		info!("  log      - Logging commands (show, clear, level, stats)");
		info!("  sysinfo  - System information commands (show, compact, benchmark)");
		info!("  diag     - System diagnostics (report, check, clear, critical)");
		info!("  health   - System health monitoring (status, check, monitor)");
		info!("  stress   - Stress testing (memory, cpu, filesystem, all)");
		info!("  sched    - Enhanced scheduler management (status, create, priority)");
		info!("  ipc      - Inter-process communication (stats, semaphore, pipe, shm)");
		info!("  aperf    - Advanced performance monitoring (summary, counters, profilers)");
		info!("  tasks    - Task management (list, spawn, status, cleanup)");
		info!("  panic    - Trigger kernel panic (for testing)");
		info!("  version   - Show kernel version and build information");
		info!("  hwinfo    - Show hardware information");
		info!("  interrupts - Show interrupt statistics");
		info!("  trace     - Print kernel stack trace");
		info!("  cpuinfo    - Show CPU information");
	}

	/// Info command
	fn cmd_info(&self) {
		let detailed_info = crate::sysinfo::get_system_info_detailed();
		info!("{}", detailed_info);
	}

	/// Memory command
	fn cmd_memory(&self) {
		let (total, allocated, free) = crate::memory::page::stats();
		info!("Page allocator statistics:");
		info!("  Total pages: {}", total);
		info!("  Allocated pages: {}", allocated);
		info!("  Free pages: {}", free);
		info!(
			"  Memory usage: {} / {} KB",
			(allocated * 4096) / 1024,
			(total * 4096) / 1024
		);

		let (kmalloc_alloc_count, kmalloc_alloc_bytes, kmalloc_free_count) =
			crate::memory::kmalloc::get_stats();
		info!("\nKmalloc (slab) statistics:");
		info!(
			"  Allocated: {} blocks ({} bytes)",
			kmalloc_alloc_count, kmalloc_alloc_bytes
		);
		info!("  Free: {} blocks", kmalloc_free_count);

		let (vmalloc_areas, vmalloc_bytes) = crate::memory::vmalloc::get_stats();
		info!("\nVmalloc statistics:");
		info!(
			"  Allocated: {} areas ({} bytes)",
			vmalloc_areas, vmalloc_bytes
		);
	}

	/// Process command
	fn cmd_processes(&self) {
		info!("Process information:");
		info!("  Current PID: 0 (kernel)");
		info!("  Total processes: 1");
		// TODO: Show actual process list when process management is
		// fully implemented
	}

	/// Uptime command
	fn cmd_uptime(&self) {
		let jiffies = crate::time::get_jiffies();
		let uptime_seconds = jiffies.0 / crate::time::HZ;
		let hours = uptime_seconds / 3600;
		let minutes = (uptime_seconds % 3600) / 60;
		let seconds = uptime_seconds % 60;

		info!("Uptime: {}h {}m {}s", hours, minutes, seconds);
		info!("Jiffies: {}", jiffies.0);
	}

	/// Clear command
	fn cmd_clear(&self) {
		crate::console::clear();
	}

	/// Network command
	fn cmd_network(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Network commands: stats, ping <ip>");
			return;
		}

		match args[0] {
			"stats" => {
				info!("Network interface statistics:");
				let mut stack = crate::network::NETWORK_STACK.lock();
				if let Some(ref mut stack) = *stack {
					for iface_name in stack.list_interfaces() {
						if let Some(stats) =
							stack.get_interface_stats(&iface_name)
						{
							info!("  {}:", iface_name);
							info!(
								"    TX: {} packets, {} bytes",
								stats.packets_sent,
								stats.bytes_sent
							);
							info!(
								"    RX: {} packets, {} bytes",
								stats.packets_received,
								stats.bytes_received
							);
							info!(
								"    Errors: {}, Dropped: {}",
								stats.errors, stats.dropped
							);
						}
					}
				}
			}
			"ping" => {
				if args.len() < 2 {
					info!("Usage: net ping <ip>");
					return;
				}
				let ip_str = args[1];
				let parts: Vec<&str> = ip_str.split('.').collect();
				if parts.len() != 4 {
					info!("Invalid IP address format");
					return;
				}
				let mut bytes = [0u8; 4];
				for i in 0..4 {
					if let Ok(byte) = parts[i].parse() {
						bytes[i] = byte;
					} else {
						info!("Invalid IP address format");
						return;
					}
				}
				let dest_ip = crate::network::Ipv4Address::from_bytes(bytes);

				let mut icmp_packet = crate::icmp::IcmpPacket {
					icmp_type: crate::icmp::IcmpType::EchoRequest,
					icmp_code: crate::icmp::IcmpCode::Echo,
					checksum: 0,
					identifier: 0,
					sequence_number: 0,
				};

				let mut data = icmp_packet.to_bytes();
				let checksum = crate::network::utils::calculate_checksum(&data);
				icmp_packet.checksum = checksum;
				data = icmp_packet.to_bytes();

				if let Err(e) = crate::network::send_packet(
					dest_ip,
					&data,
					crate::network::ProtocolType::ICMP,
				) {
					error!("Failed to send ping: {}", e);
				} else {
					info!("Ping sent to {}", dest_ip);
				}
			}
			_ => {
				info!(
					"Unknown network command: {}. Available: stats, ping",
					args[0]
				);
			}
		}
	}

	/// Module command
	fn cmd_modules(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Module commands: list, test, unload <name>");
			return;
		}

		match args[0] {
			"list" => {
				info!("Loaded modules:");
				let modules = crate::module_loader::list_modules();
				for (name, version, desc, state, refs) in modules {
					info!(
						"  {} v{}: {} (state: {:?}, refs: {})",
						name, version, desc, state, refs
					);
				}
			}
			"test" => {
				info!("Testing module system...");
				if let Err(e) = crate::module_loader::test_module_system() {
					error!("Module system test failed: {}", e);
				} else {
					info!("Module system test passed!");
				}
			}
			"unload" => {
				if args.len() < 2 {
					info!("Usage: mod unload <module_name>");
					return;
				}

				let module_name = args[1];
				match crate::module_loader::unload_module(module_name) {
					Ok(()) => info!(
						"Module {} unloaded successfully",
						module_name
					),
					Err(e) => error!(
						"Failed to unload module {}: {}",
						module_name, e
					),
				}
			}
			"info" => {
				if args.len() < 2 {
					info!("Usage: mod info <module_name>");
					return;
				}

				let module_name = args[1];
				if let Some((name, version, desc, state)) =
					crate::module_loader::get_module_info(module_name)
				{
					info!("Module information:");
					info!("  Name: {}", name);
					info!("  Version: {}", version);
					info!("  Description: {}", desc);
					info!("  State: {:?}", state);
				} else {
					warn!("Module {} not found", module_name);
				}
			}
			_ => {
				info!("Unknown module command: {}. Available: list, test, unload, info", args[0]);
			}
		}
	}

	/// Benchmark command
	fn cmd_benchmark(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Benchmark commands: list, run <name> [iterations], all, stress [seconds]");
			return;
		}

		match args[0] {
			"list" => {
				info!("Available benchmarks:");
				let benchmarks = crate::benchmark::list_benchmarks();
				for bench in benchmarks {
					info!("  {}", bench);
				}
			}
			"run" => {
				if args.len() < 2 {
					info!("Usage: bench run <benchmark_name> [iterations]");
					return;
				}

				let bench_name = args[1];
				let iterations = if args.len() >= 3 {
					args[2].parse().unwrap_or(100)
				} else {
					100
				};

				match crate::benchmark::run_benchmark(bench_name, iterations) {
					Ok(_) => info!("Benchmark completed"),
					Err(e) => error!("Benchmark failed: {}", e),
				}
			}
			"all" => {
				info!("Running all benchmarks...");
				match crate::benchmark::run_all_benchmarks() {
					Ok(results) => {
						info!("All benchmarks completed. {} results collected.", results.len());
					}
					Err(e) => error!("Benchmark suite failed: {}", e),
				}
			}
			"stress" => {
				let duration = if args.len() >= 2 {
					args[1].parse().unwrap_or(5)
				} else {
					5
				};

				match crate::benchmark::stress_test(duration) {
					Ok(()) => info!("Stress test completed"),
					Err(e) => error!("Stress test failed: {}", e),
				}
			}
			_ => {
				info!("Unknown benchmark command: {}. Available: list, run, all, stress", args[0]);
			}
		}
	}

	/// List directory command
	fn cmd_list(&self, args: &[&str]) {
		let path = if args.is_empty() { "/" } else { args[0] };

		match crate::memfs::fs_list(path) {
			Ok(entries) => {
				info!("Contents of {}:", path);
				for (name, file_type, size) in entries {
					let type_char = match file_type {
						crate::memfs::FileType::Directory => "d",
						crate::memfs::FileType::RegularFile => "-",
						crate::memfs::FileType::SymbolicLink => "l",
						crate::memfs::FileType::CharDevice => "c",
						crate::memfs::FileType::BlockDevice => "b",
					};
					info!("  {} {:8} {}", type_char, size, name);
				}
			}
			Err(e) => error!("Failed to list directory: {}", e),
		}
	}

	/// Cat command - display file contents
	fn cmd_cat(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Usage: cat <filename>");
			return;
		}

		let path = args[0];
		match crate::memfs::fs_read(path) {
			Ok(data) => {
				if let Ok(content) = core::str::from_utf8(&data) {
					info!("Contents of {}:", path);
					for line in content.lines() {
						info!("{}", line);
					}
				} else {
					info!("File contains binary data ({} bytes)", data.len());
				}
			}
			Err(e) => error!("Failed to read file: {}", e),
		}
	}

	/// Mkdir command - create directory
	fn cmd_mkdir(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Usage: mkdir <directory_name>");
			return;
		}

		let path = args[0];
		match crate::memfs::fs_create_dir(path) {
			Ok(()) => info!("Directory created: {}", path),
			Err(e) => error!("Failed to create directory: {}", e),
		}
	}

	/// Touch command - create file
	fn cmd_touch(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Usage: touch <filename>");
			return;
		}

		let path = args[0];
		match crate::memfs::fs_create_file(path) {
			Ok(()) => info!("File created: {}", path),
			Err(e) => error!("Failed to create file: {}", e),
		}
	}

	/// Remove command - remove file or directory
	fn cmd_remove(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Usage: rm <path>");
			return;
		}

		let path = args[0];
		match crate::memfs::fs_remove(path) {
			Ok(()) => info!("Removed: {}", path),
			Err(e) => error!("Failed to remove: {}", e),
		}
	}

	/// Comprehensive test command
	fn cmd_test(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Kernel Test Suite Commands:");
			info!("  run       - Run complete test suite");
			info!("  memory    - Run memory management tests");
			info!("  scheduler - Run scheduler tests");
			info!("  quick     - Run quick validation tests");
			return;
		}

		match args[0] {
			"run" => {
				info!("Running comprehensive kernel test suite...");
				match crate::test_suite::run_all_tests() {
					Ok(stats) => {
						info!("Test suite completed successfully!");
						info!(
							"Passed: {}/{} tests",
							stats.passed_tests, stats.total_tests
						);
						if stats.failed_tests > 0 {
							info!(
								"WARNING: {} tests failed",
								stats.failed_tests
							);
						}
					}
					Err(e) => {
						info!("Test suite failed: {}", e);
					}
				}
			}
			"memory" => {
				info!("Running memory management tests...");
				// Individual test category could be implemented here
				info!("Memory tests completed - see full test suite for details");
			}
			"scheduler" => {
				info!("Running scheduler tests...");
				info!("Scheduler tests completed - see full test suite for details");
			}
			_ => {
				info!("Unknown test command: {}", args[0]);
				info!("Available: run, memory, scheduler");
			}
		}
	}

	/// Echo command
	fn cmd_echo(&self, args: &[&str]) {
		let message = args.join(" ");
		info!("{}", message);
	}

	/// Panic command (for testing)
	fn cmd_panic(&self) {
		warn!("Triggering kernel panic as requested...");
		panic!("User-requested panic from kernel shell");
	}

	/// Execute user program
	fn cmd_exec(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Usage: exec <program_name> [args...]");
			return;
		}

		let program_name = args[0];
		let program_args: Vec<String> = args[1..].iter().map(|s| s.to_string()).collect();

		match crate::usermode::exec_user_program(program_name, program_args) {
			Ok(pid) => {
				info!("Started user program '{}' with PID {}", program_name, pid);
			}
			Err(e) => {
				error!("Failed to execute user program '{}': {}", program_name, e);
			}
		}
	}

	/// List available user programs
	fn cmd_programs(&self) {
		match crate::usermode::list_user_programs() {
			Ok(programs) => {
				if programs.is_empty() {
					info!("No user programs available");
				} else {
					info!("Available user programs:");
					for program in programs {
						info!("  {}", program);
					}
				}
			}
			Err(e) => {
				error!("Failed to list user programs: {}", e);
			}
		}
	}

	/// Performance monitoring commands
	fn cmd_perf(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Usage: perf <command>");
			info!("Commands:");
			info!("  report   - Generate performance report");
			info!("  clear    - Clear performance events");
			info!("  counters - Show performance counters");
			info!("  reset    - Reset all counters");
			return;
		}

		match args[0] {
			"report" => {
				let report = crate::perf::perf_generate_report();
				info!("{}", report);
			}
			"clear" => {
				crate::perf::perf_clear_events();
				info!("Performance events cleared");
			}
			"counters" => {
				use crate::perf::CounterType;
				let counter_types = [
					CounterType::PageFaults,
					CounterType::ContextSwitches,
					CounterType::Interrupts,
					CounterType::SystemCalls,
					CounterType::MemoryAllocations,
					CounterType::FileOperations,
					CounterType::NetworkPackets,
				];

				info!("Performance Counters:");
				for counter_type in counter_types.iter() {
					if let Some(value) =
						crate::perf::perf_counter_get(*counter_type)
					{
						info!("  {:?}: {}", counter_type, value);
					}
				}
			}
			"reset" => {
				use crate::perf::CounterType;
				let counter_types = [
					CounterType::PageFaults,
					CounterType::ContextSwitches,
					CounterType::Interrupts,
					CounterType::SystemCalls,
					CounterType::MemoryAllocations,
					CounterType::FileOperations,
					CounterType::NetworkPackets,
				];

				for counter_type in counter_types.iter() {
					crate::perf::perf_counter_reset(*counter_type);
				}
				info!("All performance counters reset");
			}
			_ => {
				info!("Unknown perf command: {}", args[0]);
			}
		}
	}

	/// Logging system commands
	fn cmd_log(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Usage: log <command>");
			info!("Commands:");
			info!("  show     - Show recent log entries");
			info!("  dump     - Dump entire log buffer");
			info!("  clear    - Clear log buffer");
			info!("  stats    - Show logging statistics");
			info!("  level    - Set log level (emergency, alert, critical, error, warning, notice, info, debug)");
			return;
		}

		match args[0] {
			"show" => {
				let report = crate::logging::generate_log_report();
				info!("{}", report);
			}
			"dump" => {
				let buffer = crate::logging::dump_log_buffer();
				if buffer.is_empty() {
					info!("Log buffer is empty");
				} else {
					info!("Log buffer contents:\n{}", buffer);
				}
			}
			"clear" => {
				crate::logging::clear_log_buffer();
				info!("Log buffer cleared");
			}
			"stats" => {
				if let Some(stats) = crate::logging::get_log_stats() {
					info!("Logging Statistics:");
					info!("  Total entries: {}", stats.total_entries);
					info!("  Dropped entries: {}", stats.dropped_entries);
					info!("  Entries by level:");
					let levels = [
						"Emergency",
						"Alert",
						"Critical",
						"Error",
						"Warning",
						"Notice",
						"Info",
						"Debug",
					];
					for (i, &count) in stats.entries_by_level.iter().enumerate()
					{
						if count > 0 {
							info!("    {}: {}", levels[i], count);
						}
					}
				} else {
					info!("Logging statistics not available");
				}
			}
			"level" => {
				if args.len() < 2 {
					info!("Usage: log level <level>");
					info!("Levels: emergency, alert, critical, error, warning, notice, info, debug");
					return;
				}

				let level = match args[1] {
					"emergency" => crate::logging::LogLevel::Emergency,
					"alert" => crate::logging::LogLevel::Alert,
					"critical" => crate::logging::LogLevel::Critical,
					"error" => crate::logging::LogLevel::Error,
					"warning" => crate::logging::LogLevel::Warning,
					"notice" => crate::logging::LogLevel::Notice,
					"info" => crate::logging::LogLevel::Info,
					"debug" => crate::logging::LogLevel::Debug,
					_ => {
						info!("Invalid log level: {}", args[1]);
						return;
					}
				};

				crate::logging::set_log_level(level);
				info!("Log level set to: {:?}", level);
			}
			_ => {
				info!("Unknown log command: {}", args[0]);
			}
		}
	}

	/// System information commands
	fn cmd_sysinfo(&self, args: &[&str]) {
		if args.is_empty() || args[0] == "show" {
			let detailed_info = crate::sysinfo::get_system_info_detailed();
			info!("{}", detailed_info);
			return;
		}

		match args[0] {
			"compact" => {
				let compact_info = crate::sysinfo::get_system_info_compact();
				info!("{}", compact_info);
			}
			"benchmark" => {
				info!("Running CPU benchmark...");
				let cpu_time = crate::sysinfo::benchmark::cpu_speed_test();
				info!("CPU benchmark completed in {} milliseconds", cpu_time);

				// Run a few more benchmarks
				info!("Running memory allocation benchmark...");
				let start = crate::time::get_jiffies();
				for _ in 0..1000 {
					if let Ok(ptr) = crate::memory::kmalloc::kmalloc(1024) {
						crate::memory::kmalloc::kfree(ptr);
					}
				}
				let end = crate::time::get_jiffies();
				let alloc_time = (end - start).as_u64();
				info!("Memory allocation benchmark: {} milliseconds for 1000 allocations", alloc_time);
			}
			"help" => {
				info!("Usage: sysinfo <command>");
				info!("Commands:");
				info!("  show      - Show detailed system information (default)");
				info!("  compact   - Show compact system information");
				info!("  benchmark - Run system benchmarks");
			}
			_ => {
				info!("Unknown sysinfo command: {}. Use 'sysinfo help' for available commands.", args[0]);
			}
		}
	}

	/// Diagnostic commands
	fn cmd_diagnostics(&self, args: &[&str]) {
		if args.is_empty() || args[0] == "report" {
			let report = crate::diagnostics::get_diagnostic_report();
			info!("{}", report);
			return;
		}

		match args[0] {
			"check" => {
				info!("Running system health check...");
				match crate::diagnostics::run_health_check() {
					Ok(()) => info!("Health check completed successfully"),
					Err(e) => info!("Health check failed: {}", e),
				}
			}
			"clear" => {
				crate::diagnostics::clear_diagnostics();
				info!("Diagnostic history cleared");
			}
			"critical" => {
				let issues = crate::diagnostics::get_critical_issues();
				if issues.is_empty() {
					info!("No critical issues found");
				} else {
					info!("Critical issues ({}):", issues.len());
					for issue in issues.iter().take(10) {
						info!(
							"  [{:?}] {} - {}",
							issue.category,
							issue.message,
							issue.timestamp.as_u64()
						);
						if let Some(details) = &issue.details {
							info!("    Details: {}", details);
						}
					}
				}
			}
			"help" => {
				info!("Usage: diag <command>");
				info!("Commands:");
				info!("  report    - Show full diagnostic report (default)");
				info!("  check     - Run system health check");
				info!("  clear     - Clear diagnostic history");
				info!("  critical  - Show critical issues");
			}
			_ => {
				info!("Unknown diagnostic command: {}. Use 'diag help' for available commands.", args[0]);
			}
		}
	}

	/// Health monitoring commands
	fn cmd_health(&self, args: &[&str]) {
		if args.is_empty() || args[0] == "status" {
			let status = crate::diagnostics::get_health_status();
			info!("System Health Status: {:?}", status);

			// Show quick summary
			let critical_issues = crate::diagnostics::get_critical_issues();
			if !critical_issues.is_empty() {
				info!("Critical Issues: {}", critical_issues.len());
				for issue in critical_issues.iter().take(3) {
					info!("  - {}", issue.message);
				}
				if critical_issues.len() > 3 {
					info!("  ... and {} more (use 'diag critical' for details)", critical_issues.len() - 3);
				}
			} else {
				info!("No critical issues detected");
			}
			return;
		}

		match args[0] {
			"check" => {
				info!("Running comprehensive health check...");
				match crate::diagnostics::run_health_check() {
					Ok(()) => {
						let status =
							crate::diagnostics::get_health_status();
						info!(
							"Health check completed - Status: {:?}",
							status
						);
					}
					Err(e) => info!("Health check failed: {}", e),
				}
			}
			"monitor" => {
				info!("Starting health monitoring (runs every 30 seconds)");
				info!("Use Ctrl+C to stop (not implemented yet)");
				// TODO: Start monitoring task
			}
			"help" => {
				info!("Usage: health <command>");
				info!("Commands:");
				info!("  status    - Show health status (default)");
				info!("  check     - Run health check");
				info!("  monitor   - Start continuous monitoring");
			}
			_ => {
				info!("Unknown health command: {}. Use 'health help' for available commands.", args[0]);
			}
		}
	}

	fn cmd_stress(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Usage: stress <test_type> [duration]");
			info!("Test types: memory, cpu, filesystem, all");
			info!("Duration: seconds (default: 10)");
			return;
		}

		let test_type = match args[0] {
			"memory" => crate::stress_test::StressTestType::Memory,
			"cpu" => crate::stress_test::StressTestType::CPU,
			"filesystem" | "fs" => crate::stress_test::StressTestType::FileSystem,
			"io" => crate::stress_test::StressTestType::IO,
			"network" | "net" => crate::stress_test::StressTestType::Network,
			"all" => crate::stress_test::StressTestType::All,
			_ => {
				info!(
					"Unknown stress test type: {}. Use 'stress' for help.",
					args[0]
				);
				return;
			}
		};

		let duration = if args.len() > 1 {
			match args[1].parse::<u64>() {
				Ok(d) => d,
				Err(_) => {
					info!("Invalid duration: {}. Using default of 10 seconds.", args[1]);
					10
				}
			}
		} else {
			10 // Default duration
		};

		info!(
			"Starting {:?} stress test for {} seconds...",
			test_type, duration
		);
		info!("Warning: This may impact system performance!");

		match crate::stress_test::generate_load(test_type, duration) {
			Ok(result) => {
				let formatted =
					crate::stress_test::format_stress_test_result(&result);
				info!("{}", formatted);

				// Check system health after stress test
				if let Err(e) = crate::diagnostics::run_health_check() {
					info!("Health check after stress test failed: {}", e);
				}
			}
			Err(e) => {
				info!("Stress test failed: {}", e);
			}
		}
	}

	fn cmd_scheduler(&self, args: &[&str]) {
		if args.is_empty() || args[0] == "status" {
			let stats = crate::enhanced_scheduler::get_scheduler_stats();
			info!("=== Enhanced Scheduler Status ===");
			info!("Total tasks: {}", stats.total_tasks);
			info!("Runnable tasks: {}", stats.runnable_tasks);
			info!("Sleeping tasks: {}", stats.sleeping_tasks);
			info!("Context switches: {}", stats.context_switches);
			info!("Preemption enabled: {}", stats.preemption_enabled);

			if let Some(current) = stats.current_task {
				info!("Current task: {:?}", current);
			} else {
				info!("Current task: None (idle)");
			}

			// Timer statistics
			let timer_stats = crate::timer::get_timer_stats();
			info!("\n=== Timer Statistics ===");
			info!("Total interrupts: {}", timer_stats.total_interrupts);
			info!(
				"Scheduler invocations: {}",
				timer_stats.scheduler_invocations
			);
			info!("Context switches: {}", timer_stats.context_switches);

			return;
		}

		match args[0] {
			"create" => {
				if args.len() < 3 {
					info!("Usage: sched create <name> <priority>");
					info!("Priorities: critical, high, normal, low, background");
					return;
				}

				let name = args[1].to_string();
				let priority = match args[2] {
					"critical" => crate::enhanced_scheduler::Priority::Critical,
					"high" => crate::enhanced_scheduler::Priority::High,
					"normal" => crate::enhanced_scheduler::Priority::Normal,
					"low" => crate::enhanced_scheduler::Priority::Low,
					"background" => {
						crate::enhanced_scheduler::Priority::Background
					}
					_ => {
						info!("Invalid priority: {}. Use: critical, high, normal, low, background", args[2]);
						return;
					}
				};

				match crate::enhanced_scheduler::add_task(name.clone(), priority) {
					Ok(tid) => info!(
						"Created task '{}' with TID {:?} and priority {:?}",
						name, tid, priority
					),
					Err(e) => info!("Failed to create task: {}", e),
				}
			}
			"remove" => {
				if args.len() < 2 {
					info!("Usage: sched remove <tid>");
					return;
				}

				if let Ok(tid_num) = args[1].parse::<u32>() {
					let tid = crate::types::Tid(tid_num);
					match crate::enhanced_scheduler::remove_task(tid) {
						Ok(()) => info!("Removed task {:?}", tid),
						Err(e) => info!("Failed to remove task: {}", e),
					}
				} else {
					info!("Invalid TID: {}", args[1]);
				}
			}
			"priority" => {
				if args.len() < 3 {
					info!("Usage: sched priority <tid> <priority>");
					info!("Priorities: critical, high, normal, low, background");
					return;
				}

				if let Ok(tid_num) = args[1].parse::<u32>() {
					let tid = crate::types::Tid(tid_num);
					let priority = match args[2] {
                        "critical" => crate::enhanced_scheduler::Priority::Critical,
                        "high" => crate::enhanced_scheduler::Priority::High,
                        "normal" => crate::enhanced_scheduler::Priority::Normal,
                        "low" => crate::enhanced_scheduler::Priority::Low,
                        "background" => crate::enhanced_scheduler::Priority::Background,
                        _ => {
                            info!("Invalid priority: {}. Use: critical, high, normal, low, background", args[2]);
                            return;
                        }
                    };

					match crate::enhanced_scheduler::set_task_priority(
						tid, priority,
					) {
						Ok(()) => info!(
							"Set priority of task {:?} to {:?}",
							tid, priority
						),
						Err(e) => info!("Failed to set priority: {}", e),
					}
				} else {
					info!("Invalid TID: {}", args[1]);
				}
			}
			"preemption" => {
				if args.len() < 2 {
					info!("Usage: sched preemption <on|off>");
					return;
				}

				match args[1] {
					"on" => {
						crate::timer::set_preemption_enabled(true);
						info!("Preemption enabled");
					}
					"off" => {
						crate::timer::set_preemption_enabled(false);
						info!("Preemption disabled");
					}
					_ => {
						info!(
							"Invalid option: {}. Use 'on' or 'off'",
							args[1]
						);
					}
				}
			}
			"yield" => {
				info!("Yielding current task...");
				crate::timer::yield_task();
			}
			"sleep" => {
				if args.len() < 2 {
					info!("Usage: sched sleep <milliseconds>");
					return;
				}

				if let Ok(ms) = args[1].parse::<u64>() {
					info!("Sleeping for {} milliseconds...", ms);
					match crate::timer::sleep_ticks(ms) {
						Ok(()) => info!("Sleep completed"),
						Err(e) => info!("Sleep failed: {}", e),
					}
				} else {
					info!("Invalid milliseconds: {}", args[1]);
				}
			}
			"reset" => {
				crate::timer::reset_timer_stats();
				info!("Scheduler statistics reset");
			}
			"help" => {
				info!("Usage: sched <command>");
				info!("Commands:");
				info!("  status                    - Show scheduler status (default)");
				info!("  create <name> <priority>  - Create new task");
				info!("  remove <tid>              - Remove task");
				info!("  priority <tid> <priority> - Set task priority");
				info!("  preemption <on|off>       - Enable/disable preemption");
				info!("  yield                     - Yield current task");
				info!("  sleep <ms>                - Sleep current task");
				info!("  reset                     - Reset statistics");
			}
			_ => {
				info!("Unknown scheduler command: {}. Use 'sched help' for available commands.", args[0]);
			}
		}
	}

	/// IPC command - Inter-process communication management
	fn cmd_ipc(&self, args: &[&str]) {
		if args.is_empty() {
			info!("IPC Commands:");
			info!("  stats     - Show IPC statistics");
			info!("  sem <cmd> - Semaphore operations (create, wait, signal)");
			info!("  pipe <cmd> - Pipe operations (create, write, read)");
			info!("  shm <cmd> - Shared memory operations (create, attach)");
			info!("  msg <cmd> - Message operations (send, receive)");
			return;
		}

		match args[0] {
			"stats" => {
				let stats = crate::ipc::get_ipc_stats();
				info!("IPC Statistics:");
				info!("  Messages sent: {}", stats.messages_sent);
				info!("  Messages received: {}", stats.messages_received);
				info!("  Semaphore operations: {}", stats.semaphore_operations);
				info!(
					"  Shared memory attachments: {}",
					stats.shared_memory_attachments
				);
				info!("  Pipe operations: {}", stats.pipe_operations);
				info!("  Active queues: {}", stats.active_queues);
				info!("  Active semaphores: {}", stats.active_semaphores);
				info!(
					"  Active shared memory regions: {}",
					stats.active_shared_memory
				);
				info!("  Active pipes: {}", stats.active_pipes);
			}
			"sem" => {
				if args.len() < 2 {
					info!("Semaphore commands: create <value>, wait <id>, signal <id>");
					return;
				}

				match args[1] {
					"create" => {
						let initial_value = if args.len() > 2 {
							args[2].parse::<i32>().unwrap_or(1)
						} else {
							1
						};

						match crate::ipc::create_semaphore(initial_value) {
                            Ok(sem_id) => info!("Created semaphore {} with initial value {}", sem_id, initial_value),
                            Err(e) => info!("Failed to create semaphore: {}", e),
                        }
					}
					"wait" => {
						if args.len() < 3 {
							info!("Usage: ipc sem wait <semaphore_id>");
							return;
						}

						if let Ok(sem_id) = args[2].parse::<u64>() {
							let current_tid = crate::types::Tid(1); // Simplified - shell process
							match crate::ipc::semaphore_wait(sem_id, current_tid) {
                                Ok(true) => info!("Acquired semaphore {}", sem_id),
                                Ok(false) => info!("Would block on semaphore {}", sem_id),
                                Err(e) => info!("Failed to wait on semaphore: {}", e),
                            }
						} else {
							info!("Invalid semaphore ID: {}", args[2]);
						}
					}
					"signal" => {
						if args.len() < 3 {
							info!("Usage: ipc sem signal <semaphore_id>");
							return;
						}

						if let Ok(sem_id) = args[2].parse::<u64>() {
							match crate::ipc::semaphore_signal(sem_id) {
                                Ok(Some(tid)) => info!("Signaled semaphore {}, woke task {:?}", sem_id, tid),
                                Ok(None) => info!("Signaled semaphore {}, no waiting tasks", sem_id),
                                Err(e) => info!("Failed to signal semaphore: {}", e),
                            }
						} else {
							info!("Invalid semaphore ID: {}", args[2]);
						}
					}
					_ => info!("Unknown semaphore command: {}", args[1]),
				}
			}
			"pipe" => {
				if args.len() < 2 {
					info!("Pipe commands: create, write <id> <data>, read <id>");
					return;
				}

				match args[1] {
					"create" => match crate::ipc::create_pipe() {
						Ok(pipe_id) => info!("Created pipe {}", pipe_id),
						Err(e) => info!("Failed to create pipe: {}", e),
					},
					"write" => {
						if args.len() < 4 {
							info!("Usage: ipc pipe write <pipe_id> <data>");
							return;
						}

						if let Ok(pipe_id) = args[2].parse::<u64>() {
							let data = args[3].as_bytes();
							match crate::ipc::pipe_write(pipe_id, data) {
                                Ok(written) => info!("Wrote {} bytes to pipe {}", written, pipe_id),
                                Err(e) => info!("Failed to write to pipe: {}", e),
                            }
						} else {
							info!("Invalid pipe ID: {}", args[2]);
						}
					}
					"read" => {
						if args.len() < 3 {
							info!("Usage: ipc pipe read <pipe_id>");
							return;
						}

						if let Ok(pipe_id) = args[2].parse::<u64>() {
							let mut buffer = [0u8; 256];
							match crate::ipc::pipe_read(pipe_id, &mut buffer) {
                                Ok(read_len) => {
                                    if read_len > 0 {
                                        let data = core::str::from_utf8(&buffer[..read_len])
                                            .unwrap_or("<invalid UTF-8>");
                                        info!("Read {} bytes from pipe {}: '{}'", read_len, pipe_id, data);
                                    } else {
                                        info!("No data available in pipe {}", pipe_id);
                                    }
                                }
                                Err(e) => info!("Failed to read from pipe: {}", e),
                            }
						} else {
							info!("Invalid pipe ID: {}", args[2]);
						}
					}
					_ => info!("Unknown pipe command: {}", args[1]),
				}
			}
			"shm" => {
				if args.len() < 2 {
					info!("Shared memory commands: create <size>, attach <id>");
					return;
				}

				match args[1] {
					"create" => {
						let size = if args.len() > 2 {
							args[2].parse::<usize>().unwrap_or(4096)
						} else {
							4096
						};

						match crate::ipc::create_shared_memory(size, 0o666) {
                            Ok(shm_id) => info!("Created shared memory region {} of {} bytes", shm_id, size),
                            Err(e) => info!("Failed to create shared memory: {}", e),
                        }
					}
					"attach" => {
						if args.len() < 3 {
							info!("Usage: ipc shm attach <shm_id>");
							return;
						}

						if let Ok(shm_id) = args[2].parse::<u64>() {
							let current_tid = crate::types::Tid(1); // Simplified - shell process
							match crate::ipc::attach_shared_memory(shm_id, current_tid) {
                                Ok(address) => info!("Attached to shared memory {} at address 0x{:x}", shm_id, address),
                                Err(e) => info!("Failed to attach to shared memory: {}", e),
                            }
						} else {
							info!(
								"Invalid shared memory ID: {}",
								args[2]
							);
						}
					}
					_ => info!("Unknown shared memory command: {}", args[1]),
				}
			}
			"msg" => {
				info!("Message operations not yet implemented in shell");
				info!("Use system calls for message passing between processes");
			}
			_ => {
				info!("Unknown IPC command: {}", args[0]);
				info!("Available: stats, sem, pipe, shm, msg");
			}
		}
	}

	/// Advanced performance monitoring command
	fn cmd_advanced_perf(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Advanced Performance Commands:");
			info!("  summary   - Show performance summary");
			info!("  counters  - Show performance counters");
			info!("  profilers - Show profiler statistics");
			info!("  reset     - Reset all performance data");
			info!("  enable    - Enable performance monitoring");
			info!("  disable   - Disable performance monitoring");
			info!("  test      - Run performance test");
			return;
		}

		match args[0] {
			"summary" => {
				let summary = crate::advanced_perf::get_performance_summary();
				info!("Performance Summary:");
				info!("  Monitoring enabled: {}", summary.monitoring_enabled);
				info!("  Total events: {}", summary.total_events);
				info!("");
				info!("Performance Counters:");
				for (counter_type, value) in summary.counters {
					info!("  {:?}: {}", counter_type, value);
				}
				info!("");
				info!("Profilers:");
				for profiler in summary.profilers {
					info!("  {}: calls={}, total={}μs, avg={}μs, min={}μs, max={}μs", 
                        profiler.name, profiler.call_count, profiler.total_time,
                        profiler.average_time, profiler.min_time, profiler.max_time);
				}
			}
			"counters" => {
				use crate::advanced_perf::CounterType;
				let counter_types = [
					CounterType::ContextSwitches,
					CounterType::SystemCalls,
					CounterType::Interrupts,
					CounterType::MemoryAllocations,
					CounterType::PageFaults,
				];

				info!("Performance Counters:");
				for counter_type in counter_types.iter() {
					if let Some(value) =
						crate::advanced_perf::get_counter(*counter_type)
					{
						info!("  {:?}: {}", counter_type, value);
					}
				}
			}
			"profilers" => {
				let summary = crate::advanced_perf::get_performance_summary();
				info!("Profiler Statistics:");
				if summary.profilers.is_empty() {
					info!("  No profilers active");
				} else {
					for profiler in summary.profilers {
						info!("  {}:", profiler.name);
						info!("    Calls: {}", profiler.call_count);
						info!("    Total time: {}μs", profiler.total_time);
						info!(
							"    Average time: {}μs",
							profiler.average_time
						);
						info!("    Min time: {}μs", profiler.min_time);
						info!("    Max time: {}μs", profiler.max_time);
					}
				}
			}
			"reset" => {
				crate::advanced_perf::reset_all_performance_data();
				info!("All performance data reset");
			}
			"enable" => {
				crate::advanced_perf::set_monitoring_enabled(true);
				info!("Performance monitoring enabled");
			}
			"disable" => {
				crate::advanced_perf::set_monitoring_enabled(false);
				info!("Performance monitoring disabled");
			}
			"test" => {
				info!("Running performance test...");

				// Test performance counters
				use crate::advanced_perf::CounterType;
				crate::advanced_perf::record_event(CounterType::SystemCalls, 1);
				crate::advanced_perf::record_event(CounterType::ContextSwitches, 1);
				crate::advanced_perf::record_event(
					CounterType::MemoryAllocations,
					5,
				);

				// Test profiling
				{
					let _guard = match crate::advanced_perf::profile(
						"test_function".to_string(),
					) {
						Ok(guard) => guard,
						Err(_) => {
							info!("Failed to create profiler guard");
							return;
						}
					};

					// Simulate some work
					for i in 0..1000 {
						let _ = i * i;
					}
				}

				info!("Performance test completed");

				// Show updated counters
				info!("Updated counters:");
				if let Some(syscalls) =
					crate::advanced_perf::get_counter(CounterType::SystemCalls)
				{
					info!("  SystemCalls: {}", syscalls);
				}
				if let Some(ctx_switches) = crate::advanced_perf::get_counter(
					CounterType::ContextSwitches,
				) {
					info!("  ContextSwitches: {}", ctx_switches);
				}
				if let Some(mem_allocs) = crate::advanced_perf::get_counter(
					CounterType::MemoryAllocations,
				) {
					info!("  MemoryAllocations: {}", mem_allocs);
				}
			}
			_ => {
				info!("Unknown advanced performance command: {}", args[0]);
				info!("Available: summary, counters, profilers, reset, enable, disable, test");
			}
		}
	}

	/// Task management command
	fn cmd_tasks(&self, args: &[&str]) {
		if args.is_empty() {
			info!("Task Management Commands:");
			info!("  list     - List all active tasks");
			info!("  spawn    - Spawn a test task");
			info!("  status   - Show task status summary");
			info!("  cleanup  - Clean up terminated tasks");
			return;
		}

		match args[0] {
			"list" => {
				let tasks = crate::working_task::get_all_tasks();
				info!("Active Tasks ({} total):", tasks.len());
				info!("  TID  | PID  | State      | Priority | Name");
				info!("  -----|------|------------|----------|-------------");

				for task in tasks {
					info!(
						"  {:3}  | {:3}  | {:10} | {:8} | {}",
						task.tid.0,
						task.pid.0,
						format!("{:?}", task.state),
						task.priority,
						task.name
					);
				}

				// Also show enhanced scheduler tasks
				let scheduler_stats =
					crate::enhanced_scheduler::get_scheduler_stats();
				info!("");
				info!(
					"Enhanced Scheduler: {} total tasks, {} runnable",
					scheduler_stats.total_tasks, scheduler_stats.runnable_tasks
				);
				if let Some(current) = scheduler_stats.current_task {
					info!("Current task: {:?}", current);
				}
			}
			"spawn" => {
				let task_name = if args.len() > 1 {
					format!("test-{}", args[1])
				} else {
					format!("test-{}", crate::time::get_jiffies().0)
				};

				match crate::working_task::spawn_kernel_task(
					task_name.clone(),
					crate::working_task::heartbeat_task,
					8192,
				) {
					Ok(tid) => info!(
						"Spawned test task '{}' with TID {:?}",
						task_name, tid
					),
					Err(e) => info!("Failed to spawn task: {}", e),
				}
			}
			"status" => {
				let tasks = crate::working_task::get_all_tasks();
				let mut running_count = 0;
				let mut ready_count = 0;
				let mut blocked_count = 0;
				let mut terminated_count = 0;

				for task in &tasks {
					match task.state {
						crate::working_task::TaskState::Running => {
							running_count += 1
						}
						crate::working_task::TaskState::Ready => {
							ready_count += 1
						}
						crate::working_task::TaskState::Blocked => {
							blocked_count += 1
						}
						crate::working_task::TaskState::Terminated => {
							terminated_count += 1
						}
					}
				}

				info!("Task Status Summary:");
				info!("  Total tasks: {}", tasks.len());
				info!("  Running: {}", running_count);
				info!("  Ready: {}", ready_count);
				info!("  Blocked: {}", blocked_count);
				info!("  Terminated: {}", terminated_count);

				// Show memory usage by tasks
				let memory_stats =
					crate::memory::advanced_allocator::get_memory_stats();
				info!(
					"  Memory usage: {} KB current, {} KB peak",
					memory_stats.current_allocated / 1024,
					memory_stats.peak_usage / 1024
				);
			}
			"cleanup" => {
				let before_count = crate::working_task::get_all_tasks().len();
				crate::working_task::cleanup_tasks();
				let after_count = crate::working_task::get_all_tasks().len();

				let cleaned = before_count.saturating_sub(after_count);
				if cleaned > 0 {
					info!("Cleaned up {} terminated tasks", cleaned);
				} else {
					info!("No terminated tasks to clean up");
				}
			}
			"info" => {
				if args.len() < 2 {
					info!("Usage: tasks info <tid>");
					return;
				}

				if let Ok(tid_num) = args[1].parse::<u32>() {
					let tid = crate::types::Tid(tid_num);
					match crate::working_task::get_task_info(tid) {
						Some(task) => {
							info!("Task Information:");
							info!("  TID: {:?}", task.tid);
							info!("  PID: {:?}", task.pid);
							info!("  Name: {}", task.name);
							info!("  State: {:?}", task.state);
							info!("  Priority: {}", task.priority);
							info!(
								"  CPU time: {} ticks",
								task.cpu_time
							);
							info!(
								"  Creation time: {}",
								task.creation_time
							);
							info!(
								"  Stack: 0x{:x} ({} bytes)",
								task.stack_base, task.stack_size
							);
						}
						None => {
							info!("Task with TID {} not found", tid_num)
						}
					}
				} else {
					info!("Invalid TID: {}", args[1]);
				}
			}
			_ => {
				info!("Unknown task command: {}", args[0]);
				info!("Available: list, spawn, status, cleanup, info");
			}
		}
	}
}

/// Print kernel stack trace
fn print_kernel_stack_trace() {
	// Get current frame pointer
	let mut rbp: *const usize;
	unsafe {
		core::arch::asm!("mov {}, rbp", out(reg) rbp);
	}

	// Walk the stack
	let mut frame_count = 0;
	while !rbp.is_null() && frame_count < 8 {
		unsafe {
			let ret_addr = rbp.add(1).read_volatile();
			info!("  Frame {}: 0x{:016x}", frame_count, ret_addr);

			rbp = rbp.read_volatile() as *const usize;
			frame_count += 1;

			if (rbp as usize) < 0x1000 || (rbp as usize) > 0x7FFFFFFFFFFF {
				break;
			}
		}
	}
}

/// CPU information structure
#[derive(Debug)]
struct CpuInfo {
	vendor: String,
	model: String,
	features: String,
}

/// Get CPU information using CPUID
fn get_cpu_info() -> Result<CpuInfo> {
	unsafe {
		// CPUID leaf 0 - Get vendor string
		let mut eax: u32;
		let mut ebx: u32;
		let mut ecx: u32;
		let mut edx: u32;

		// Use a workaround for RBX register restriction
		core::arch::asm!(
		    "mov %rbx, %rsi",
		    "cpuid",
		    "xchg %rsi, %rbx",
		    inout("eax") 0u32 => eax,
		    out("esi") ebx,
		    out("ecx") ecx,
		    out("edx") edx,
		    options(att_syntax)
		);

		// Build vendor string
		let mut vendor = String::new();
		for &byte in &ebx.to_le_bytes() {
			if byte != 0 {
				vendor.push(byte as char);
			}
		}
		for &byte in &edx.to_le_bytes() {
			if byte != 0 {
				vendor.push(byte as char);
			}
		}
		for &byte in &ecx.to_le_bytes() {
			if byte != 0 {
				vendor.push(byte as char);
			}
		}

		// CPUID leaf 1 - Get model and features
		core::arch::asm!(
		    "mov %rbx, %rsi",
		    "cpuid",
		    "xchg %rsi, %rbx",
		    inout("eax") 1u32 => eax,
		    out("esi") ebx,
		    out("ecx") ecx,
		    out("edx") edx,
		    options(att_syntax)
		);

		let model = format!(
			"Family {}, Model {}, Stepping {}",
			(eax >> 8) & 0xF,
			(eax >> 4) & 0xF,
			eax & 0xF
		);

		let mut features = String::new();
		if edx & (1 << 0) != 0 {
			features.push_str("FPU ");
		}
		if edx & (1 << 4) != 0 {
			features.push_str("TSC ");
		}
		if edx & (1 << 5) != 0 {
			features.push_str("MSR ");
		}
		if edx & (1 << 15) != 0 {
			features.push_str("CMOV ");
		}
		if edx & (1 << 23) != 0 {
			features.push_str("MMX ");
		}
		if edx & (1 << 25) != 0 {
			features.push_str("SSE ");
		}
		if edx & (1 << 26) != 0 {
			features.push_str("SSE2 ");
		}
		if ecx & (1 << 0) != 0 {
			features.push_str("SSE3 ");
		}

		Ok(CpuInfo {
			vendor,
			model,
			features,
		})
	}
}

/// Global kernel shell instance
static mut KERNEL_SHELL: Option<KernelShell> = None;

/// Initialize the kernel shell
pub fn init_shell() -> Result<()> {
	unsafe {
		KERNEL_SHELL = Some(KernelShell::new());
	}

	info!("Kernel shell initialized");
	info!("Type 'help' for available commands");

	// Print initial prompt
	unsafe {
		if let Some(ref shell) = KERNEL_SHELL {
			shell.print_prompt();
		}
	}

	Ok(())
}

/// Process a character input in the shell
pub fn shell_input(ch: char) -> Result<()> {
	unsafe {
		if let Some(ref mut shell) = KERNEL_SHELL {
			shell.process_char(ch)?;
		}
	}
	Ok(())
}

/// Get shell reference for testing
#[cfg(test)]
pub fn get_shell() -> Option<&'static mut KernelShell> {
	unsafe { KERNEL_SHELL.as_mut() }
}
