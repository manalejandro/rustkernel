// SPDX-License-Identifier: GPL-2.0

//! Kernel shell - a simple command-line interface

use crate::error::Result;
use crate::{info, warn, error};
use alloc::{string::{String, ToString}, vec::Vec};

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
                "panic" => self.cmd_panic(),
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
        info!("  panic    - Trigger kernel panic (for testing)");
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
        
        // TODO: Add more memory statistics (kmalloc, vmalloc, etc.)
    }
    
    /// Process command
    fn cmd_processes(&self) {
        info!("Process information:");
        info!("  Current PID: 0 (kernel)");
        info!("  Total processes: 1");
        // TODO: Show actual process list when process management is fully implemented
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
        // TODO: Clear console screen
        info!("Clear screen not implemented yet");
    }
    
    /// Network command
    fn cmd_network(&self, args: &[&str]) {
        if args.is_empty() {
            info!("Network commands: stats, test");
            return;
        }
        
        match args[0] {
            "stats" => {
                info!("Network interface statistics:");
                if let Some(stats) = crate::net_basic::get_net_stats("lo") {
                    info!("  lo (loopback):");
                    info!("    TX: {} packets, {} bytes", stats.tx_packets, stats.tx_bytes);
                    info!("    RX: {} packets, {} bytes", stats.rx_packets, stats.rx_bytes);
                    info!("    Errors: TX {}, RX {}", stats.tx_errors, stats.rx_errors);
                } else {
                    warn!("Failed to get loopback statistics");
                }
            }
            "test" => {
                info!("Running network tests...");
                if let Err(e) = crate::net_basic::test_networking() {
                    error!("Network tests failed: {}", e);
                } else {
                    info!("Network tests passed!");
                }
            }
            _ => {
                info!("Unknown network command: {}. Available: stats, test", args[0]);
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
                    info!("  {} v{}: {} (state: {:?}, refs: {})", name, version, desc, state, refs);
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
                    Ok(()) => info!("Module {} unloaded successfully", module_name),
                    Err(e) => error!("Failed to unload module {}: {}", module_name, e),
                }
            }
            "info" => {
                if args.len() < 2 {
                    info!("Usage: mod info <module_name>");
                    return;
                }
                
                let module_name = args[1];
                if let Some((name, version, desc, state)) = crate::module_loader::get_module_info(module_name) {
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
    
    /// Test command
    fn cmd_test(&self, args: &[&str]) {
        if args.is_empty() {
            info!("Running basic kernel tests...");
            if let Err(e) = crate::test_init::run_basic_tests() {
                error!("Tests failed: {}", e);
            } else {
                info!("All tests passed!");
            }
        } else {
            match args[0] {
                "memory" => {
                    info!("Running memory tests...");
                    if let Err(e) = crate::test_init::test_memory_management() {
                        error!("Memory tests failed: {}", e);
                    } else {
                        info!("Memory tests passed!");
                    }
                }
                "interrupt" => {
                    info!("Testing interrupt handling...");
                    // Disable and enable interrupts
                    crate::interrupt::disable();
                    info!("Interrupts disabled");
                    crate::interrupt::enable();
                    info!("Interrupts enabled");
                }
                "fs" => {
                    info!("Testing file system...");
                    // Create a test file
                    if let Ok(()) = crate::memfs::fs_create_file("/test.txt") {
                        // Write some data
                        if let Ok(_) = crate::memfs::fs_write("/test.txt", b"Hello from file system!") {
                            // Read it back
                            if let Ok(data) = crate::memfs::fs_read("/test.txt") {
                                if let Ok(content) = core::str::from_utf8(&data) {
                                    info!("File system test passed: {}", content);
                                } else {
                                    error!("File system test failed: invalid UTF-8");
                                }
                            } else {
                                error!("File system test failed: read error");
                            }
                        } else {
                            error!("File system test failed: write error");
                        }
                        // Clean up
                        let _ = crate::memfs::fs_remove("/test.txt");
                    } else {
                        error!("File system test failed: create error");
                    }
                }
                _ => {
                    info!("Unknown test: {}. Available: memory, interrupt, fs", args[0]);
                }
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
                    if let Some(value) = crate::perf::perf_counter_get(*counter_type) {
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
                    let levels = ["Emergency", "Alert", "Critical", "Error", "Warning", "Notice", "Info", "Debug"];
                    for (i, &count) in stats.entries_by_level.iter().enumerate() {
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
                        info!("  [{:?}] {} - {}", issue.category, issue.message, issue.timestamp.as_u64());
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
                        let status = crate::diagnostics::get_health_status();
                        info!("Health check completed - Status: {:?}", status);
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
                info!("Unknown stress test type: {}. Use 'stress' for help.", args[0]);
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
        
        info!("Starting {:?} stress test for {} seconds...", test_type, duration);
        info!("Warning: This may impact system performance!");
        
        match crate::stress_test::generate_load(test_type, duration) {
            Ok(result) => {
                let formatted = crate::stress_test::format_stress_test_result(&result);
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
