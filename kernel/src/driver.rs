// SPDX-License-Identifier: GPL-2.0

//! Driver framework compatible with Linux kernel

use alloc::{
	boxed::Box,
	collections::BTreeMap,
	string::{String, ToString},
	vec::Vec,
};

use crate::device::Device;
use crate::error::{Error, Result};
use crate::sync::Spinlock; // Add ToString

/// Driver trait - Linux compatible
pub trait Driver: Send + Sync + core::fmt::Debug {
	/// Driver name
	fn name(&self) -> &str;

	/// Probe function - called when device is found
	fn probe(&self, device: &mut Device) -> Result<()>;

	/// Remove function - called when device is removed
	fn remove(&self, device: &mut Device) -> Result<()>;

	/// Suspend function - power management
	fn suspend(&self, device: &mut Device) -> Result<()> {
		// Default implementation does nothing
		Ok(())
	}

	/// Resume function - power management
	fn resume(&self, device: &mut Device) -> Result<()> {
		// Default implementation does nothing
		Ok(())
	}

	/// Shutdown function - system shutdown
	fn shutdown(&self, device: &mut Device) {
		// Default implementation does nothing
	}
}

/// Driver operations for character devices
pub trait CharDriverOps: Send + Sync {
	fn open(&self, inode: &crate::device::Inode, file: &mut crate::device::File) -> Result<()>;
	fn release(
		&self,
		inode: &crate::device::Inode,
		file: &mut crate::device::File,
	) -> Result<()>;
	fn read(
		&self,
		file: &mut crate::device::File,
		buf: &mut [u8],
		offset: u64,
	) -> Result<usize>;
	fn write(&self, file: &mut crate::device::File, buf: &[u8], offset: u64) -> Result<usize>;
	fn ioctl(&self, file: &mut crate::device::File, cmd: u32, arg: usize) -> Result<usize>;
}

/// Driver operations for block devices
pub trait BlockDriverOps: Send + Sync {
	fn read_block(&self, block: u64, buffer: &mut [u8]) -> Result<usize>;
	fn write_block(&self, block: u64, buffer: &[u8]) -> Result<usize>;
	fn get_block_size(&self) -> u32;
	fn get_total_blocks(&self) -> u64;
	fn flush(&self) -> Result<()>;
}

/// Platform driver - for platform devices
pub trait PlatformDriver: Driver {
	/// Match function - check if driver supports device
	fn match_device(&self, device: &Device) -> bool;

	/// Get supported device IDs
	fn device_ids(&self) -> &[DeviceId];
}

/// Device ID structure - for driver matching
#[derive(Debug, Clone)]
pub struct DeviceId {
	pub name: String,
	pub vendor_id: Option<u32>,
	pub device_id: Option<u32>,
	pub class: Option<u32>,
	pub compatible: Vec<String>, // Device tree compatible strings
}

impl DeviceId {
	pub fn new(name: String) -> Self {
		Self {
			name,
			vendor_id: None,
			device_id: None,
			class: None,
			compatible: Vec::new(),
		}
	}

	pub fn with_vendor_device(mut self, vendor_id: u32, device_id: u32) -> Self {
		self.vendor_id = Some(vendor_id);
		self.device_id = Some(device_id);
		self
	}

	pub fn with_compatible(mut self, compatible: Vec<String>) -> Self {
		self.compatible = compatible;
		self
	}
}

/// PCI driver - for PCI devices  
pub trait PciDriver: Driver {
	/// PCI device IDs supported by this driver
	fn pci_ids(&self) -> &[PciDeviceId];

	/// PCI-specific probe
	fn pci_probe(&self, pci_dev: &mut PciDevice) -> Result<()>;

	/// PCI-specific remove
	fn pci_remove(&self, pci_dev: &mut PciDevice) -> Result<()>;
}

/// PCI device ID
#[derive(Debug, Clone, Copy)]
pub struct PciDeviceId {
	pub vendor: u16,
	pub device: u16,
	pub subvendor: u16,
	pub subdevice: u16,
	pub class: u32,
	pub class_mask: u32,
}

impl PciDeviceId {
	pub const fn new(vendor: u16, device: u16) -> Self {
		Self {
			vendor,
			device,
			subvendor: 0xFFFF, // PCI_ANY_ID
			subdevice: 0xFFFF,
			class: 0,
			class_mask: 0,
		}
	}
}

/// PCI device structure
#[derive(Debug, Clone)]
pub struct PciDevice {
	pub vendor: u16,
	pub device: u16,
	pub subsystem_vendor: u16,
	pub subsystem_device: u16,
	pub class: u32,
	pub revision: u8,
	pub bus: u8,
	pub slot: u8,
	pub function: u8,
	pub irq: u32,
	pub bars: [PciBar; 6],
}

/// PCI Base Address Register
#[derive(Debug, Clone, Copy)]
pub struct PciBar {
	pub address: u64,
	pub size: u64,
	pub flags: u32,
}

impl PciBar {
	pub fn new() -> Self {
		Self {
			address: 0,
			size: 0,
			flags: 0,
		}
	}

	pub fn is_io(&self) -> bool {
		self.flags & 1 != 0
	}

	pub fn is_memory(&self) -> bool {
		!self.is_io()
	}

	pub fn is_64bit(&self) -> bool {
		self.is_memory() && (self.flags & 0x6) == 0x4
	}
}

/// USB driver - for USB devices
pub trait UsbDriver: Driver {
	/// USB device IDs supported by this driver
	fn usb_ids(&self) -> &[UsbDeviceId];

	/// USB-specific probe
	fn usb_probe(&self, usb_dev: &mut UsbDevice) -> Result<()>;

	/// USB-specific disconnect
	fn usb_disconnect(&self, usb_dev: &mut UsbDevice) -> Result<()>;
}

/// USB device ID
#[derive(Debug, Clone, Copy)]
pub struct UsbDeviceId {
	pub vendor: u16,
	pub product: u16,
	pub device_class: u8,
	pub device_subclass: u8,
	pub device_protocol: u8,
	pub interface_class: u8,
	pub interface_subclass: u8,
	pub interface_protocol: u8,
}

/// USB device structure
#[derive(Debug)]
pub struct UsbDevice {
	pub vendor: u16,
	pub product: u16,
	pub device_class: u8,
	pub device_subclass: u8,
	pub device_protocol: u8,
	pub speed: UsbSpeed,
	pub address: u8,
	pub configuration: u8,
}

/// USB speeds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
	Low,       // 1.5 Mbps
	Full,      // 12 Mbps
	High,      // 480 Mbps
	Super,     // 5 Gbps
	SuperPlus, // 10 Gbps
}

/// Global driver subsystem
static DRIVER_SUBSYSTEM: Spinlock<DriverSubsystem> = Spinlock::new(DriverSubsystem::new());

/// Driver subsystem state
struct DriverSubsystem {
	drivers: BTreeMap<String, Box<dyn Driver>>,
	platform_drivers: Vec<Box<dyn PlatformDriver>>,
	pci_drivers: Vec<Box<dyn PciDriver>>,
	usb_drivers: Vec<Box<dyn UsbDriver>>,
}

impl DriverSubsystem {
	const fn new() -> Self {
		Self {
			drivers: BTreeMap::new(),
			platform_drivers: Vec::new(),
			pci_drivers: Vec::new(),
			usb_drivers: Vec::new(),
		}
	}

	fn register_driver(&mut self, driver: Box<dyn Driver>) -> Result<()> {
		let name = driver.name().to_string();
		if self.drivers.contains_key(&name) {
			return Err(Error::Busy);
		}
		self.drivers.insert(name, driver);
		Ok(())
	}

	fn unregister_driver(&mut self, name: &str) -> Result<()> {
		if self.drivers.remove(name).is_some() {
			Ok(())
		} else {
			Err(Error::NotFound)
		}
	}

	#[allow(dead_code)]
	fn find_driver(&self, name: &str) -> Option<&dyn Driver> {
		self.drivers.get(name).map(|d| d.as_ref())
	}
}

/// Register a driver
pub fn register_driver(driver: Box<dyn Driver>) -> Result<()> {
	let mut subsystem = DRIVER_SUBSYSTEM.lock();
	let name = driver.name().to_string();
	subsystem.register_driver(driver)?;
	crate::info!("Registered driver: {}", name);
	Ok(())
}

/// Unregister a driver
pub fn unregister_driver(name: &str) -> Result<()> {
	let mut subsystem = DRIVER_SUBSYSTEM.lock();
	subsystem.unregister_driver(name)?;
	crate::info!("Unregistered driver: {}", name);
	Ok(())
}

/// Register a platform driver
pub fn register_platform_driver(driver: Box<dyn PlatformDriver>) -> Result<()> {
	let mut subsystem = DRIVER_SUBSYSTEM.lock();
	let name = driver.name().to_string();

	// Also register as a regular driver
	let driver_copy = unsafe {
		// This is a bit of a hack - we need to clone the driver
		// In a real implementation, we'd use Arc or similar
		core::mem::transmute::<*const dyn PlatformDriver, Box<dyn Driver>>(
			driver.as_ref() as *const dyn PlatformDriver
		)
	};
	subsystem.register_driver(driver_copy)?;
	subsystem.platform_drivers.push(driver);

	crate::info!("Registered platform driver: {}", name);
	Ok(())
}

/// Register a PCI driver
pub fn register_pci_driver(driver: Box<dyn PciDriver>) -> Result<()> {
	let mut subsystem = DRIVER_SUBSYSTEM.lock();
	let name = driver.name().to_string();
	subsystem.pci_drivers.push(driver);
	crate::info!("Registered PCI driver: {}", name);
	Ok(())
}

/// Register a USB driver
pub fn register_usb_driver(driver: Box<dyn UsbDriver>) -> Result<()> {
	let mut subsystem = DRIVER_SUBSYSTEM.lock();
	let name = driver.name().to_string();
	subsystem.usb_drivers.push(driver);
	crate::info!("Registered USB driver: {}", name);
	Ok(())
}

/// Find and match a driver for a device
pub fn match_driver(device: &Device) -> Option<String> {
	let subsystem = DRIVER_SUBSYSTEM.lock();

	// Try platform drivers first
	for driver in &subsystem.platform_drivers {
		if driver.match_device(device) {
			return Some(driver.name().to_string());
		}
	}

	// TODO: Try PCI, USB, etc. drivers based on device type

	None
}

/// Get list of registered drivers
pub fn list_drivers() -> Vec<String> {
	let subsystem = DRIVER_SUBSYSTEM.lock();
	subsystem.drivers.keys().cloned().collect()
}

/// Module macros for easier driver registration
#[macro_export]
macro_rules! platform_driver {
	($driver:ident) => {
		#[no_mangle]
		pub extern "C" fn init_module() -> core::ffi::c_int {
			match $crate::driver::register_platform_driver(Box::new($driver)) {
				Ok(()) => 0,
				Err(e) => e.to_errno(),
			}
		}

		#[no_mangle]
		pub extern "C" fn cleanup_module() {
			$crate::driver::unregister_driver(stringify!($driver)).ok();
		}
	};
}

pub fn pci_config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
	crate::hardware::pci_config_read(bus, device, function, offset)
}

#[macro_export]
macro_rules! pci_driver {
	($driver:ident) => {
		#[no_mangle]
		pub extern "C" fn init_module() -> core::ffi::c_int {
			match $crate::driver::register_pci_driver(Box::new($driver)) {
				Ok(()) => 0,
				Err(e) => e.to_errno(),
			}
		}

		#[no_mangle]
		pub extern "C" fn cleanup_module() {
			$crate::driver::unregister_driver(stringify!($driver)).ok();
		}
	};
}
