// SPDX-License-Identifier: GPL-2.0

//! Test kernel module

#![no_std]
#![no_main]

use kernel::prelude::*;

struct TestModule {
    test_data: Vec<u32>,
    counter: u32,
}

impl kernel::module::Module for TestModule {
    fn init(_module: &'static kernel::module::ThisModule) -> Result<Self> {
        info!("Test module initializing...");
        
        // Test memory allocation
        let mut test_data = Vec::new();
        for i in 0..10 {
            test_data.push(i * i);
        }
        
        info!("Test data created: {:?}", test_data);
        
        // Test synchronization
        let spinlock = Spinlock::new(42);
        {
            let guard = spinlock.lock();
            info!("Locked value: {}", *guard);
        }
        
        info!("Test module initialized successfully");
        
        Ok(TestModule {
            test_data,
            counter: 0,
        })
    }
    
    fn exit(_module: &'static kernel::module::ThisModule) {
        info!("Test module exiting, counter was: {}", self.counter);
    }
}

module! {
    type: TestModule,
    name: "test_module", 
    author: "Rust Kernel Contributors",
    description: "A test module for kernel functionality",
    license: "GPL-2.0",
}
