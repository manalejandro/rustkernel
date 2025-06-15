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
    
    // TODO: Initialize drivers
    // init_drivers();
    
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
