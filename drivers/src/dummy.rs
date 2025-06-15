// SPDX-License-Identifier: GPL-2.0

//! Dummy device driver

#![no_std]
#![no_main]

use kernel::prelude::*;
use kernel::driver::{Driver, DriverOps};
use kernel::device::{Device, DeviceType};

struct DummyDriver {
    name: &'static str,
}

impl DummyDriver {
    fn new() -> Self {
        Self {
            name: "dummy_driver",
        }
    }
}

impl Driver for DummyDriver {
    fn name(&self) -> &str {
        self.name
    }
    
    fn probe(&self, device: &mut Device) -> Result<()> {
        info!("DummyDriver: probing device {}", device.name);
        Ok(())
    }
    
    fn remove(&self, device: &mut Device) -> Result<()> {
        info!("DummyDriver: removing device {}", device.name);
        Ok(())
    }
}

impl DriverOps for DummyDriver {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        info!("DummyDriver: read at offset {} for {} bytes", offset, buffer.len());
        // Fill buffer with dummy data
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        Ok(buffer.len())
    }
    
    fn write(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        info!("DummyDriver: write at offset {} for {} bytes", offset, buffer.len());
        Ok(buffer.len())
    }
    
    fn ioctl(&self, cmd: u32, arg: usize) -> Result<usize> {
        info!("DummyDriver: ioctl cmd={}, arg={}", cmd, arg);
        Ok(0)
    }
}

struct DummyModule {
    driver: DummyDriver,
}

impl kernel::module::Module for DummyModule {
    fn init(_module: &'static kernel::module::ThisModule) -> Result<Self> {
        info!("Dummy driver module initializing...");
        
        let driver = DummyDriver::new();
        
        // Register the driver
        kernel::driver::register_driver(Box::new(driver))?;
        
        info!("Dummy driver registered successfully");
        
        Ok(DummyModule {
            driver: DummyDriver::new(),
        })
    }
    
    fn exit(_module: &'static kernel::module::ThisModule) {
        info!("Dummy driver module exiting");
        // Unregister driver
        kernel::driver::unregister_driver("dummy_driver").ok();
    }
}

module! {
    type: DummyModule,
    name: "dummy_driver",
    author: "Rust Kernel Contributors", 
    description: "A dummy device driver for testing",
    license: "GPL-2.0",
}
