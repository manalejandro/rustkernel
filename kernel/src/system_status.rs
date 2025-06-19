// SPDX-License-Identifier: GPL-2.0

//! System overview and status reporting

use crate::error::Result;
use alloc::string::String;

/// Comprehensive system status report
pub fn get_system_status() -> String {
    let mut status = String::new();
    
    status.push_str("=== RUST KERNEL SYSTEM STATUS ===\n");
    status.push_str("\n");
    
    // Kernel version and build info
    status.push_str("Kernel Information:\n");
    status.push_str("  Version: Rust Kernel v0.1.0\n");
    status.push_str("  Architecture: x86_64\n");
    status.push_str("  Build: Advanced Features Edition\n");
    status.push_str("\n");
    
    // System uptime
    let uptime = crate::time::get_jiffies().0;
    status.push_str(&format!("System Uptime: {} ticks\n\n", uptime));
    
    // Memory status
    let memory_stats = crate::memory::advanced_allocator::get_memory_stats();
    status.push_str("Memory Management:\n");
    status.push_str(&format!("  Current allocated: {} KB\n", memory_stats.current_allocated / 1024));
    status.push_str(&format!("  Peak usage: {} KB\n", memory_stats.peak_usage / 1024));
    status.push_str(&format!("  Total allocations: {}\n", memory_stats.allocation_count));
    status.push_str(&format!("  Active allocations: {}\n", memory_stats.active_allocations));
    status.push_str("\n");
    
    // Task management
    let tasks = crate::working_task::get_all_tasks();
    let scheduler_stats = crate::enhanced_scheduler::get_scheduler_stats();
    status.push_str("Task Management:\n");
    status.push_str(&format!("  Active tasks: {}\n", tasks.len()));
    status.push_str(&format!("  Scheduler tasks: {}\n", scheduler_stats.total_tasks));
    status.push_str(&format!("  Runnable tasks: {}\n", scheduler_stats.runnable_tasks));
    status.push_str(&format!("  Context switches: {}\n", scheduler_stats.context_switches));
    status.push_str(&format!("  Preemption enabled: {}\n", scheduler_stats.preemption_enabled));
    if let Some(current) = scheduler_stats.current_task {
        status.push_str(&format!("  Current task: {:?}\n", current));
    }
    status.push_str("\n");
    
    // IPC status
    let ipc_stats = crate::ipc::get_ipc_stats();
    status.push_str("Inter-Process Communication:\n");
    status.push_str(&format!("  Messages sent: {}\n", ipc_stats.messages_sent));
    status.push_str(&format!("  Messages received: {}\n", ipc_stats.messages_received));
    status.push_str(&format!("  Semaphore operations: {}\n", ipc_stats.semaphore_operations));
    status.push_str(&format!("  Shared memory attachments: {}\n", ipc_stats.shared_memory_attachments));
    status.push_str(&format!("  Pipe operations: {}\n", ipc_stats.pipe_operations));
    status.push_str(&format!("  Active queues: {}\n", ipc_stats.active_queues));
    status.push_str(&format!("  Active semaphores: {}\n", ipc_stats.active_semaphores));
    status.push_str(&format!("  Active pipes: {}\n", ipc_stats.active_pipes));
    status.push_str("\n");
    
    // Performance monitoring
    let perf_summary = crate::advanced_perf::get_performance_summary();
    status.push_str("Performance Monitoring:\n");
    status.push_str(&format!("  Monitoring enabled: {}\n", perf_summary.monitoring_enabled));
    status.push_str(&format!("  Total events: {}\n", perf_summary.total_events));
    status.push_str(&format!("  Active profilers: {}\n", perf_summary.profilers.len()));
    
    // Show key performance counters
    for (counter_type, value) in &perf_summary.counters {
        if *value > 0 {
            status.push_str(&format!("  {:?}: {}\n", counter_type, value));
        }
    }
    status.push_str("\n");
    
    // System diagnostics
    let diag_report = crate::diag::get_diagnostics_report();
    status.push_str("System Health:\n");
    status.push_str(&format!("  Total checks: {}\n", diag_report.total_checks));
    status.push_str(&format!("  Issues found: {}\n", diag_report.issues_found));
    status.push_str(&format!("  Critical issues: {}\n", diag_report.critical_issues));
    status.push_str(&format!("  Health score: {:.1}%\n", diag_report.health_score));
    status.push_str("\n");
    
    // Available shell commands
    status.push_str("Available Shell Commands:\n");
    status.push_str("  Core: help, info, mem, ps, uptime, clear\n");
    status.push_str("  Files: ls, cat, mkdir, touch, rm\n");
    status.push_str("  System: sysinfo, diag, health, stress, perf\n");
    status.push_str("  Advanced: sched, ipc, aperf, tasks\n");
    status.push_str("  Testing: test, bench, mod, exec\n");
    status.push_str("  Network: net\n");
    status.push_str("  Logging: log\n");
    status.push_str("\n");
    
    // Kernel features
    status.push_str("Kernel Features:\n");
    status.push_str("  ✓ Advanced memory allocator with tracking\n");
    status.push_str("  ✓ Enhanced preemptive scheduler\n");
    status.push_str("  ✓ Timer-based interrupts and preemption\n");
    status.push_str("  ✓ Inter-process communication (IPC)\n");
    status.push_str("  ✓ Advanced performance monitoring\n");
    status.push_str("  ✓ Working kernel task management\n");
    status.push_str("  ✓ System diagnostics and health monitoring\n");
    status.push_str("  ✓ Comprehensive shell interface\n");
    status.push_str("  ✓ Exception handling and interrupt management\n");
    status.push_str("  ✓ Virtual file system with multiple implementations\n");
    status.push_str("  ✓ Device driver framework\n");
    status.push_str("  ✓ Network stack foundation\n");
    status.push_str("  ✓ System call infrastructure\n");
    status.push_str("  ✓ Process and thread management\n");
    status.push_str("  ✓ Stress testing and benchmarking\n");
    status.push_str("\n");
    
    status.push_str("=== END SYSTEM STATUS ===");
    
    status
}

/// Quick system health check
pub fn quick_health_check() -> Result<String> {
    let mut report = String::new();
    
    // Check memory
    let memory_stats = crate::memory::advanced_allocator::get_memory_stats();
    let memory_usage_percent = if memory_stats.peak_usage > 0 {
        (memory_stats.current_allocated * 100) / memory_stats.peak_usage
    } else {
        0
    };
    
    report.push_str("Quick Health Check:\n");
    
    // Memory health
    if memory_usage_percent < 80 {
        report.push_str("  Memory: ✓ Healthy\n");
    } else if memory_usage_percent < 95 {
        report.push_str("  Memory: ⚠ Warning - High usage\n");
    } else {
        report.push_str("  Memory: ✗ Critical - Very high usage\n");
    }
    
    // Task health
    let tasks = crate::working_task::get_all_tasks();
    let running_tasks = tasks.iter().filter(|t| t.state == crate::working_task::TaskState::Running).count();
    let ready_tasks = tasks.iter().filter(|t| t.state == crate::working_task::TaskState::Ready).count();
    
    if running_tasks + ready_tasks > 0 {
        report.push_str("  Tasks: ✓ Healthy\n");
    } else {
        report.push_str("  Tasks: ⚠ Warning - No active tasks\n");
    }
    
    // Scheduler health
    let sched_stats = crate::enhanced_scheduler::get_scheduler_stats();
    if sched_stats.preemption_enabled && sched_stats.runnable_tasks > 0 {
        report.push_str("  Scheduler: ✓ Healthy\n");
    } else {
        report.push_str("  Scheduler: ⚠ Warning - Issues detected\n");
    }
    
    // System diagnostics
    let diag_report = crate::diag::get_diagnostics_report();
    if diag_report.critical_issues == 0 {
        report.push_str("  Diagnostics: ✓ No critical issues\n");
    } else {
        report.push_str(&format!("  Diagnostics: ✗ {} critical issues found\n", diag_report.critical_issues));
    }
    
    Ok(report)
}

/// Get kernel feature summary
pub fn get_feature_summary() -> String {
    let mut summary = String::new();
    
    summary.push_str("Rust Kernel - Advanced Features Summary:\n\n");
    
    summary.push_str("Memory Management:\n");
    summary.push_str("  • Advanced allocator with debugging and leak detection\n");
    summary.push_str("  • Statistics tracking and performance monitoring\n");
    summary.push_str("  • Fragmentation detection and memory profiling\n\n");
    
    summary.push_str("Process Management:\n");
    summary.push_str("  • Enhanced preemptive scheduler with priorities\n");
    summary.push_str("  • Working kernel task implementation\n");
    summary.push_str("  • Context switching and CPU time tracking\n");
    summary.push_str("  • Timer-based preemption\n\n");
    
    summary.push_str("Inter-Process Communication:\n");
    summary.push_str("  • Message passing with priorities\n");
    summary.push_str("  • Semaphores for synchronization\n");
    summary.push_str("  • Shared memory regions\n");
    summary.push_str("  • Named pipes for data streaming\n\n");
    
    summary.push_str("Performance Monitoring:\n");
    summary.push_str("  • Hardware performance counters\n");
    summary.push_str("  • Function and code block profiling\n");
    summary.push_str("  • Real-time event tracking\n");
    summary.push_str("  • Automatic timing with RAII guards\n\n");
    
    summary.push_str("System Infrastructure:\n");
    summary.push_str("  • Comprehensive shell interface with 25+ commands\n");
    summary.push_str("  • System diagnostics and health monitoring\n");
    summary.push_str("  • Stress testing and benchmarking\n");
    summary.push_str("  • Virtual file system with multiple implementations\n");
    summary.push_str("  • Device driver framework\n");
    summary.push_str("  • Network stack foundation\n");
    summary.push_str("  • Exception handling and interrupt management\n\n");
    
    summary.push_str("This kernel demonstrates advanced operating system concepts\n");
    summary.push_str("implemented in safe Rust with modern design patterns.\n");
    
    summary
}
