// SPDX-License-Identifier: GPL-2.0

//! Platform device driver example

#![no_std]
#![no_main]

use kernel::prelude::*;
use kernel::driver::{PlatformDriver, Driver, DeviceId};
use kernel::device::Device;

/// Example platform driver
struct ExamplePlatformDriver {
    name: &'static str,
    device_ids: &'static [DeviceId],
}

impl ExamplePlatformDriver {
    fn new() -> Self {
        Self {
            name: "example_platform",
            device_ids: &[
                DeviceId::new(String::from("example,platform-device"))
                    .with_compatible(vec![
                        String::from("example,platform-device"),
                        String::from("generic,platform-device"),
                    ]),
            ],
        }
    }
}

impl Driver for ExamplePlatformDriver {
    fn name(&self) -> &str {
        self.name
    }
    
    fn probe(&self, device: &mut Device) -> Result<()> {
        info!("Platform driver probing device: {}", device.name());
        
        // Initialize device-specific data
        device.set_private_data(ExampleDeviceData {
            initialized: true,
            counter: 0,
        });
        
        info!("Platform device {} probed successfully", device.name());
        Ok(())
    }
    
    fn remove(&self, device: &mut Device) -> Result<()> {
        info!("Platform driver removing device: {}", device.name());
        
        if let Some(data) = device.get_private_data::<ExampleDeviceData>() {
            info!("Device had counter value: {}", data.counter);
        }
        
        Ok(())
    }
    
    fn suspend(&self, device: &mut Device) -> Result<()> {
        info!("Platform device {} suspending", device.name());
        Ok(())
    }
    
    fn resume(&self, device: &mut Device) -> Result<()> {
        info!("Platform device {} resuming", device.name());
        Ok(())
    }
    
    fn shutdown(&self, device: &mut Device) {
        info!("Platform device {} shutting down", device.name());
    }
}

impl PlatformDriver for ExamplePlatformDriver {
    fn match_device(&self, device: &Device) -> bool {
        // Simple name-based matching
        device.name().contains("example") || device.name().contains("platform")
    }
    
    fn device_ids(&self) -> &[DeviceId] {
        self.device_ids
    }
}

/// Device-specific private data
#[derive(Debug)]
struct ExampleDeviceData {
    initialized: bool,
    counter: u32,
}

/// Platform driver module
struct PlatformDriverModule {
    driver: ExamplePlatformDriver,
}

impl kernel::module::Module for PlatformDriverModule {
    fn init(_module: &'static kernel::module::ThisModule) -> Result<Self> {
        info!("Platform driver module initializing...");
        
        let driver = ExamplePlatformDriver::new();
        
        // Register the platform driver
        kernel::driver::register_platform_driver(Box::new(driver))?;
        
        info!("Platform driver registered successfully");
        
        Ok(PlatformDriverModule {
            driver: ExamplePlatformDriver::new(),
        })
    }
    
    fn exit(_module: &'static kernel::module::ThisModule) {
        info!("Platform driver module exiting");
        kernel::driver::unregister_driver("example_platform").ok();
    }
}

module! {
    type: PlatformDriverModule,
    name: "example_platform_driver",
    author: "Rust Kernel Contributors",
    description: "Example platform device driver",
    license: "GPL-2.0",
}
