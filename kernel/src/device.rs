// SPDX-License-Identifier: GPL-2.0

//! Device management compatible with Linux kernel

use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
use core::any::Any;

use crate::driver::Driver;
use crate::error::{Error, Result};
// Forward declarations for FileOperations trait
use crate::fs::{File as VfsFile, Inode as VfsInode};
use crate::memory::VmaArea;
use crate::sync::Spinlock;

/// Device number (major and minor) - Linux compatible
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceNumber {
	pub major: u32,
	pub minor: u32,
}

impl DeviceNumber {
	/// Create a new device number
	pub fn new(major: u32, minor: u32) -> Self {
		Self { major, minor }
	}

	/// Convert to raw device number (Linux dev_t equivalent)
	pub fn to_raw(&self) -> u64 {
		((self.major as u64) << 32) | (self.minor as u64)
	}

	/// Alias for to_raw for compatibility
	pub fn as_raw(&self) -> u64 {
		self.to_raw()
	}

	/// Create from raw device number
	pub fn from_raw(dev: u64) -> Self {
		Self {
			major: (dev >> 32) as u32,
			minor: (dev & 0xFFFFFFFF) as u32,
		}
	}
}

/// Linux MKDEV macro equivalent
pub fn mkdev(major: u32, minor: u32) -> DeviceNumber {
	DeviceNumber::new(major, minor)
}

/// Device types - Linux compatible
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
	Character,
	Block,
	Network,
	Input,
	Sound,
	Video,
	Misc,
	Platform,
	Pci,
	Usb,
}

/// Device structure - similar to Linux struct device
#[derive(Debug)]
pub struct Device {
	pub name: String,
	pub device_type: DeviceType,
	pub major: u32,
	pub minor: u32,
	pub driver: Option<Box<dyn Driver>>,
	pub parent: Option<String>,
	pub private_data: Option<Box<dyn Any + Send + Sync>>,
	pub power_state: PowerState,
	pub dma_coherent: bool,
	pub numa_node: i32,
}

/// Device power states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
	On,
	Suspend,
	Hibernate,
	Off,
}

impl Device {
	pub fn new(name: String, device_type: DeviceType, major: u32, minor: u32) -> Self {
		Self {
			name,
			device_type,
			major,
			minor,
			driver: None,
			parent: None,
			private_data: None,
			power_state: PowerState::On,
			dma_coherent: false,
			numa_node: -1,
		}
	}

	/// Set device driver
	pub fn set_driver(&mut self, driver: Box<dyn Driver>) -> Result<()> {
		// Probe the device with the driver
		driver.probe(self)?;
		self.driver = Some(driver);
		Ok(())
	}

	/// Remove device driver
	pub fn remove_driver(&mut self) -> Result<()> {
		if let Some(driver) = self.driver.take() {
			driver.remove(self)?;
		}
		Ok(())
	}

	/// Get device name
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Check if device has driver
	pub fn has_driver(&self) -> bool {
		self.driver.is_some()
	}

	/// Set private data
	pub fn set_private_data<T: Any + Send + Sync>(&mut self, data: T) {
		self.private_data = Some(Box::new(data));
	}

	/// Get private data
	pub fn get_private_data<T: Any + Send + Sync>(&self) -> Option<&T> {
		self.private_data.as_ref()?.downcast_ref::<T>()
	}
	/// Power management
	pub fn suspend(&mut self) -> Result<()> {
		if let Some(driver) = &self.driver {
			// We need to clone the driver to avoid borrowing issues
			// In a real implementation, we'd use Rc/Arc or other shared ownership
			// For now, we'll implement this differently
			self.power_state = PowerState::Suspend;
			// TODO: Call driver suspend when we have proper
			// ownership model
		}
		Ok(())
	}

	pub fn resume(&mut self) -> Result<()> {
		if let Some(driver) = &self.driver {
			// We need to clone the driver to avoid borrowing issues
			// In a real implementation, we'd use Rc/Arc or other shared ownership
			// For now, we'll implement this differently
			self.power_state = PowerState::On;
			// TODO: Call driver resume when we have proper
			// ownership model
		}
		Ok(())
	}
}

/// Character device structure - Linux compatible
#[derive(Debug)]
pub struct CharDevice {
	pub major: u32,
	pub minor_start: u32,
	pub minor_count: u32,
	pub name: String,
	pub fops: Option<Box<dyn FileOperations>>,
}

impl CharDevice {
	pub fn new(major: u32, minor_start: u32, minor_count: u32, name: String) -> Self {
		Self {
			major,
			minor_start,
			minor_count,
			name,
			fops: None,
		}
	}
}

/// Block device structure - Linux compatible  
#[derive(Debug)]
pub struct BlockDevice {
	pub major: u32,
	pub minor: u32,
	pub name: String,
	pub size: u64, // Size in bytes
	pub block_size: u32,
}

impl BlockDevice {
	pub fn new(major: u32, minor: u32, name: String, size: u64, block_size: u32) -> Self {
		Self {
			major,
			minor,
			name,
			size,
			block_size,
		}
	}
}

/// File operations structure - Linux compatible
pub trait FileOperations: Send + Sync + core::fmt::Debug {
	fn open(&self, inode: &VfsInode, file: &mut VfsFile) -> Result<()>;
	fn release(&self, inode: &VfsInode, file: &mut VfsFile) -> Result<()>;
	fn read(&self, file: &mut VfsFile, buf: &mut [u8], offset: u64) -> Result<usize>;
	fn write(&self, file: &mut VfsFile, buf: &[u8], offset: u64) -> Result<usize>;
	fn ioctl(&self, file: &mut VfsFile, cmd: u32, arg: usize) -> Result<usize>;
	fn mmap(&self, file: &mut VfsFile, vma: &mut VmaArea) -> Result<()>;
}

/// Re-exports for compatibility with driver.rs
pub use crate::fs::{File, Inode};

/// Global device subsystem
static DEVICE_SUBSYSTEM: Spinlock<DeviceSubsystem> = Spinlock::new(DeviceSubsystem::new());

/// Device subsystem state
struct DeviceSubsystem {
	devices: BTreeMap<String, Device>,
	char_devices: BTreeMap<u32, CharDevice>, // major -> CharDevice
	block_devices: BTreeMap<u32, BlockDevice>, // major -> BlockDevice
	next_major: u32,
}

impl DeviceSubsystem {
	const fn new() -> Self {
		Self {
			devices: BTreeMap::new(),
			char_devices: BTreeMap::new(),
			block_devices: BTreeMap::new(),
			next_major: 240, // Start with dynamic major numbers
		}
	}

	fn register_device(&mut self, device: Device) -> Result<()> {
		let name = device.name.clone();
		if self.devices.contains_key(&name) {
			return Err(Error::Busy);
		}
		self.devices.insert(name, device);
		Ok(())
	}

	fn unregister_device(&mut self, name: &str) -> Result<Device> {
		self.devices.remove(name).ok_or(Error::NotFound)
	}

	#[allow(dead_code)]
	fn find_device(&self, name: &str) -> Option<&Device> {
		self.devices.get(name)
	}

	#[allow(dead_code)]
	fn find_device_mut(&mut self, name: &str) -> Option<&mut Device> {
		self.devices.get_mut(name)
	}

	fn allocate_major(&mut self) -> u32 {
		let major = self.next_major;
		self.next_major += 1;
		major
	}
}

/// Initialize device subsystem
pub fn init() -> Result<()> {
	let mut subsystem = DEVICE_SUBSYSTEM.lock();

	// Register standard character devices
	register_std_char_devices(&mut subsystem)?;

	// Register standard block devices
	register_std_block_devices(&mut subsystem)?;

	crate::info!("Device subsystem initialized");
	Ok(())
}

/// Register standard character devices
fn register_std_char_devices(subsystem: &mut DeviceSubsystem) -> Result<()> {
	// /dev/null (major 1, minor 3)
	let null_dev = CharDevice::new(1, 3, 1, String::from("null"));
	subsystem.char_devices.insert(1, null_dev);

	// /dev/zero (major 1, minor 5)
	let zero_dev = CharDevice::new(1, 5, 1, String::from("zero"));
	// Note: This would overwrite the previous entry, so we need a better structure
	// For now, simplified

	// /dev/random (major 1, minor 8)
	let random_dev = CharDevice::new(1, 8, 1, String::from("random"));

	// /dev/urandom (major 1, minor 9)
	let urandom_dev = CharDevice::new(1, 9, 1, String::from("urandom"));

	Ok(())
}

/// Register standard block devices
fn register_std_block_devices(subsystem: &mut DeviceSubsystem) -> Result<()> {
	// RAM disk (major 1)
	let ramdisk = BlockDevice::new(1, 0, String::from("ram0"), 16 * 1024 * 1024, 4096);
	subsystem.block_devices.insert(1, ramdisk);

	Ok(())
}

/// Register a device
pub fn register_device(device: Device) -> Result<()> {
	let mut subsystem = DEVICE_SUBSYSTEM.lock();
	subsystem.register_device(device)
}

/// Unregister a device
pub fn unregister_device(name: &str) -> Result<Device> {
	let mut subsystem = DEVICE_SUBSYSTEM.lock();
	subsystem.unregister_device(name)
}

/// Find a device by name
pub fn find_device(name: &str) -> Option<&'static Device> {
	// TODO: This is unsafe and needs proper lifetime management
	// For now, we'll return None to avoid the Clone issue
	None
}

/// Register a character device
pub fn register_chrdev(major: u32, name: String, fops: Box<dyn FileOperations>) -> Result<u32> {
	let mut subsystem = DEVICE_SUBSYSTEM.lock();

	let actual_major = if major == 0 {
		subsystem.allocate_major()
	} else {
		major
	};

	let mut char_dev = CharDevice::new(actual_major, 0, 256, name);
	char_dev.fops = Some(fops);

	if subsystem.char_devices.contains_key(&actual_major) {
		return Err(Error::Busy);
	}

	subsystem.char_devices.insert(actual_major, char_dev);
	Ok(actual_major)
}

/// Unregister a character device
pub fn unregister_chrdev(major: u32) -> Result<()> {
	let mut subsystem = DEVICE_SUBSYSTEM.lock();

	if subsystem.char_devices.remove(&major).is_some() {
		Ok(())
	} else {
		Err(Error::NotFound)
	}
}

/// List all devices
pub fn list_devices() -> Vec<String> {
	let subsystem = DEVICE_SUBSYSTEM.lock();
	subsystem.devices.keys().cloned().collect()
}

/// Device tree node - for device tree support
#[derive(Debug)]
pub struct DeviceTreeNode {
	pub name: String,
	pub compatible: Vec<String>,
	pub reg: Vec<u64>,
	pub interrupts: Vec<u32>,
	pub properties: BTreeMap<String, Vec<u8>>,
}

impl DeviceTreeNode {
	pub fn new(name: String) -> Self {
		Self {
			name,
			compatible: Vec::new(),
			reg: Vec::new(),
			interrupts: Vec::new(),
			properties: BTreeMap::new(),
		}
	}
}
