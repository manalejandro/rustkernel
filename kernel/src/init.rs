// SPDX-License-Identifier: GPL-2.0

//! Kernel initialization

use crate::{info, error};

/// Early kernel initialization
pub fn early_init() {
    info!("Starting Rust Kernel v{}", crate::VERSION);
    info!("Early initialization phase");
    
    // Initialize basic console output
    if let Err(e) = crate::console::init() {
        // Can't print error since console isn't initialized yet
        loop {}
    }
    
    info!("Console initialized");
}

/// Main kernel initialization  
pub fn main_init() -> ! {
    info!("Main initialization phase");
    
    // Initialize memory management
    if let Err(e) = crate::memory::init() {
        error!("Failed to initialize memory management: {}", e);
        panic!("Memory initialization failed");
    }
    info!("Memory management initialized");
    
    // Initialize kmalloc
    if let Err(e) = crate::memory::kmalloc::init() {
        error!("Failed to initialize kmalloc: {}", e);
        panic!("Kmalloc initialization failed");
    }
    info!("Kmalloc initialized");
    
    // Initialize vmalloc
    if let Err(e) = crate::memory::vmalloc::init() {
        error!("Failed to initialize vmalloc: {}", e);
        panic!("Vmalloc initialization failed");
    }
    info!("Vmalloc initialized");
    
    // Initialize interrupt handling
    if let Err(e) = crate::interrupt::init() {
        error!("Failed to initialize interrupts: {}", e);
        panic!("Interrupt initialization failed");
    }
    info!("Interrupt handling initialized");
    
    // Initialize scheduler
    if let Err(e) = crate::scheduler::init() {
        error!("Failed to initialize scheduler: {}", e);
        panic!("Scheduler initialization failed");
    }
    info!("Scheduler initialized");
    
    // Initialize device subsystem
    if let Err(e) = crate::device::init() {
        error!("Failed to initialize devices: {}", e);
        panic!("Device initialization failed");
    }
    info!("Device subsystem initialized");
    
    // Initialize VFS (Virtual File System)
    if let Err(e) = crate::fs::init() {
        error!("Failed to initialize VFS: {}", e);
        panic!("VFS initialization failed");
    }
    info!("VFS initialized");
    
    // Initialize process management
    if let Err(e) = crate::process::init_process_management() {
        error!("Failed to initialize process management: {}", e);
        panic!("Process management initialization failed");
    }
    info!("Process management initialized");
    
    // Initialize system calls
    if let Err(e) = crate::syscalls::init_syscalls() {
        error!("Failed to initialize syscalls: {}", e);
        panic!("Syscall initialization failed");
    }
    info!("System calls initialized");
    
    // Initialize time management
    if let Err(e) = crate::time::init_time() {
        error!("Failed to initialize time management: {}", e);
        panic!("Time management initialization failed");
    }
    info!("Time management initialized");
    
    // Display system information
    crate::test_init::display_system_info();
    
    // Run basic functionality tests
    if let Err(e) = crate::test_init::run_basic_tests() {
        error!("Basic functionality tests failed: {}", e);
        panic!("Basic tests failed");
    }
    
    // Run initialization tests
    if let Err(e) = crate::test_init::run_init_tests() {
        error!("Initialization tests failed: {}", e);
        panic!("Initialization tests failed");
    }
    
    // Initialize drivers
    if let Err(e) = crate::drivers_init::init_drivers() {
        error!("Failed to initialize drivers: {}", e);
        panic!("Driver initialization failed");
    }
    info!("Drivers initialized");
    
    // Initialize kernel threads
    if let Err(e) = crate::kthread::init_kthreads() {
        error!("Failed to initialize kernel threads: {}", e);
        panic!("Kernel thread initialization failed");
    }
    info!("Kernel threads initialized");
    
    // Initialize kernel shell
    if let Err(e) = crate::shell::init_shell() {
        error!("Failed to initialize kernel shell: {}", e);
        panic!("Shell initialization failed");
    }
    info!("Kernel shell initialized");
    
    // Initialize basic networking
    if let Err(e) = crate::net_basic::init_networking() {
        error!("Failed to initialize networking: {}", e);
        panic!("Networking initialization failed");  
    }
    info!("Basic networking initialized");
    
    // Initialize module loading system
    if let Err(e) = crate::module_loader::init_modules() {
        error!("Failed to initialize module system: {}", e);
        panic!("Module system initialization failed");
    }
    info!("Module system initialized");
    
    // Initialize in-memory file system
    if let Err(e) = crate::memfs::init_memfs() {
        error!("Failed to initialize file system: {}", e);
        panic!("File system initialization failed");
    }
    info!("In-memory file system initialized");
    
    // Initialize user mode support
    if let Err(e) = crate::usermode::init_usermode() {
        error!("Failed to initialize user mode: {}", e);
        panic!("User mode initialization failed");
    }
    info!("User mode support initialized");
    
    // Initialize performance monitoring
    if let Err(e) = crate::perf::init_perf_monitor() {
        error!("Failed to initialize performance monitoring: {}", e);
        panic!("Performance monitoring initialization failed");
    }
    info!("Performance monitoring initialized");
    
    // Initialize advanced logging system
    if let Err(e) = crate::logging::init_logging() {
        error!("Failed to initialize logging system: {}", e);
        panic!("Logging system initialization failed");
    }
    info!("Advanced logging system initialized");
    
    // Initialize system information collection
    if let Err(e) = crate::sysinfo::init_sysinfo() {
        error!("Failed to initialize system information: {}", e);
        panic!("System information initialization failed");
    }
    info!("System information collection initialized");
    
    // Initialize system diagnostics
    if let Err(e) = crate::diagnostics::init_diagnostics() {
        error!("Failed to initialize system diagnostics: {}", e);
        panic!("System diagnostics initialization failed");
    }
    info!("System diagnostics initialized");
    
    // TODO: Start kernel threads
    // start_kernel_threads();

    info!("Kernel initialization completed");
    info!("Starting idle loop");
    
    // Start the idle loop
    idle_loop();
}

/// Kernel idle loop
fn idle_loop() -> ! {
    loop {
        // TODO: Power management - halt CPU until interrupt
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        core::hint::spin_loop();
        
        // TODO: Check for scheduled tasks
        // TODO: Handle background tasks
    }
}
