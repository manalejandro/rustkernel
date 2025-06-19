// SPDX-License-Identifier: GPL-2.0

//! Advanced device driver framework

use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
use core::fmt;

use crate::error::{Error, Result};
use crate::sync::Spinlock;
use crate::types::DeviceId;

/// Device class identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeviceClass {
	Block,
	Character,
	Network,
	Storage,
	Input,
	Display,
	Audio,
	USB,
	PCI,
	Platform,
	Virtual,
}

/// Device capabilities
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
	pub can_read: bool,
	pub can_write: bool,
	pub can_seek: bool,
	pub can_mmap: bool,
	pub can_poll: bool,
	pub is_removable: bool,
	pub is_hotplug: bool,
	pub supports_dma: bool,
}

impl Default for DeviceCapabilities {
	fn default() -> Self {
		Self {
			can_read: true,
			can_write: true,
			can_seek: false,
			can_mmap: false,
			can_poll: false,
			is_removable: false,
			is_hotplug: false,
			supports_dma: false,
		}
	}
}

/// Device power states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
	On,
	Standby,
	Suspend,
	Off,
}

/// PCI device information
#[derive(Debug, Clone)]
pub struct PciDeviceInfo {
	pub vendor_id: u16,
	pub device_id: u16,
	pub class_code: u8,
	pub subclass: u8,
	pub prog_if: u8,
	pub revision: u8,
	pub bus: u8,
	pub device: u8,
	pub function: u8,
	pub base_addresses: [u32; 6],
	pub irq: u8,
}

/// USB device information
#[derive(Debug, Clone)]
pub struct UsbDeviceInfo {
	pub vendor_id: u16,
	pub product_id: u16,
	pub device_class: u8,
	pub device_subclass: u8,
	pub device_protocol: u8,
	pub speed: UsbSpeed,
	pub address: u8,
	pub configuration: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum UsbSpeed {
	Low,       // 1.5 Mbps
	Full,      // 12 Mbps
	High,      // 480 Mbps
	Super,     // 5 Gbps
	SuperPlus, // 10 Gbps
}

/// Device tree information (for embedded systems)
#[derive(Debug, Clone)]
pub struct DeviceTreeInfo {
	pub compatible: Vec<String>,
	pub reg: Vec<u64>,
	pub interrupts: Vec<u32>,
	pub clocks: Vec<u32>,
	pub properties: BTreeMap<String, String>,
}

/// Advanced device structure
pub struct AdvancedDevice {
	pub id: DeviceId,
	pub name: String,
	pub class: DeviceClass,
	pub capabilities: DeviceCapabilities,
	pub power_state: PowerState,
	pub parent: Option<DeviceId>,
	pub children: Vec<DeviceId>,

	// Hardware-specific information
	pub pci_info: Option<PciDeviceInfo>,
	pub usb_info: Option<UsbDeviceInfo>,
	pub dt_info: Option<DeviceTreeInfo>,

	// Driver binding
	pub driver: Option<Box<dyn AdvancedDeviceDriver>>,
	pub driver_data: Option<Box<dyn core::any::Any + Send + Sync>>,

	// Resource management
	pub io_ports: Vec<(u16, u16)>,       // (start, size)
	pub memory_regions: Vec<(u64, u64)>, // (base, size)
	pub irq_lines: Vec<u32>,
	pub dma_channels: Vec<u32>,

	// Statistics
	pub bytes_read: u64,
	pub bytes_written: u64,
	pub error_count: u64,
	pub last_access: u64,
}

impl fmt::Debug for AdvancedDevice {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("AdvancedDevice")
			.field("id", &self.id)
			.field("name", &self.name)
			.field("class", &self.class)
			.field("capabilities", &self.capabilities)
			.field("power_state", &self.power_state)
			.field("parent", &self.parent)
			.field("children", &self.children)
			.finish()
	}
}

impl AdvancedDevice {
	pub fn new(id: DeviceId, name: String, class: DeviceClass) -> Self {
		Self {
			id,
			name,
			class,
			capabilities: DeviceCapabilities::default(),
			power_state: PowerState::Off,
			parent: None,
			children: Vec::new(),
			pci_info: None,
			usb_info: None,
			dt_info: None,
			driver: None,
			driver_data: None,
			io_ports: Vec::new(),
			memory_regions: Vec::new(),
			irq_lines: Vec::new(),
			dma_channels: Vec::new(),
			bytes_read: 0,
			bytes_written: 0,
			error_count: 0,
			last_access: 0,
		}
	}

	pub fn set_pci_info(&mut self, info: PciDeviceInfo) {
		self.pci_info = Some(info);
	}

	pub fn set_usb_info(&mut self, info: UsbDeviceInfo) {
		self.usb_info = Some(info);
	}

	pub fn add_io_port(&mut self, start: u16, size: u16) {
		self.io_ports.push((start, size));
	}

	pub fn add_memory_region(&mut self, base: u64, size: u64) {
		self.memory_regions.push((base, size));
	}

	pub fn add_irq(&mut self, irq: u32) {
		self.irq_lines.push(irq);
	}

	pub fn set_power_state(&mut self, state: PowerState) -> Result<()> {
		// Handle power state transitions
		let result = match state {
			PowerState::On => {
				// Extract driver temporarily to avoid borrow conflicts
				if let Some(mut driver) = self.driver.take() {
					let result = driver.resume(self);
					self.driver = Some(driver);
					result
				} else {
					Ok(())
				}
			}
			PowerState::Off => {
				// Extract driver temporarily to avoid borrow conflicts
				if let Some(mut driver) = self.driver.take() {
					let result = driver.suspend(self);
					self.driver = Some(driver);
					result
				} else {
					Ok(())
				}
			}
			_ => Ok(()),
		};

		if result.is_ok() {
			self.power_state = state;
		}
		result
	}

	pub fn bind_driver(&mut self, driver: Box<dyn AdvancedDeviceDriver>) -> Result<()> {
		if let Err(e) = driver.probe(self) {
			return Err(e);
		}
		self.driver = Some(driver);
		Ok(())
	}

	pub fn unbind_driver(&mut self) -> Result<()> {
		if let Some(driver) = self.driver.take() {
			driver.remove(self)?;
		}
		Ok(())
	}
}

/// Advanced device driver trait
pub trait AdvancedDeviceDriver: Send + Sync {
	fn probe(&self, device: &mut AdvancedDevice) -> Result<()>;
	fn remove(&self, device: &mut AdvancedDevice) -> Result<()>;
	fn suspend(&self, device: &mut AdvancedDevice) -> Result<()>;
	fn resume(&self, device: &mut AdvancedDevice) -> Result<()>;

	// Optional methods
	fn read(
		&self,
		_device: &mut AdvancedDevice,
		_buf: &mut [u8],
		_offset: u64,
	) -> Result<usize> {
		Err(Error::NotSupported)
	}

	fn write(&self, _device: &mut AdvancedDevice, _buf: &[u8], _offset: u64) -> Result<usize> {
		Err(Error::NotSupported)
	}

	fn ioctl(&self, _device: &mut AdvancedDevice, _cmd: u32, _arg: usize) -> Result<usize> {
		Err(Error::NotSupported)
	}

	fn interrupt_handler(&self, _device: &mut AdvancedDevice, _irq: u32) -> Result<()> {
		Ok(())
	}
}

/// Device registry for advanced devices
pub struct AdvancedDeviceRegistry {
	devices: BTreeMap<DeviceId, AdvancedDevice>,
	next_id: u32,
	drivers: Vec<Box<dyn AdvancedDeviceDriver>>,
	device_classes: BTreeMap<DeviceClass, Vec<DeviceId>>,
}

impl AdvancedDeviceRegistry {
	const fn new() -> Self {
		Self {
			devices: BTreeMap::new(),
			next_id: 1,
			drivers: Vec::new(),
			device_classes: BTreeMap::new(),
		}
	}

	pub fn register_device(&mut self, mut device: AdvancedDevice) -> Result<DeviceId> {
		let id = DeviceId(self.next_id);
		self.next_id += 1;
		device.id = id;

		// Try to bind a compatible driver
		for driver in &self.drivers {
			if device.driver.is_none() {
				if let Ok(_) = driver.probe(&mut device) {
					crate::info!("Driver bound to device {}", device.name);
					break;
				}
			}
		}

		// Add to class index
		self.device_classes
			.entry(device.class)
			.or_insert_with(Vec::new)
			.push(id);

		self.devices.insert(id, device);
		Ok(id)
	}

	pub fn unregister_device(&mut self, id: DeviceId) -> Result<()> {
		if let Some(mut device) = self.devices.remove(&id) {
			device.unbind_driver()?;

			// Remove from class index
			if let Some(devices) = self.device_classes.get_mut(&device.class) {
				devices.retain(|&x| x != id);
			}
		}
		Ok(())
	}

	pub fn register_driver(&mut self, driver: Box<dyn AdvancedDeviceDriver>) {
		// Try to bind to existing devices
		for device in self.devices.values_mut() {
			if device.driver.is_none() {
				if let Ok(_) = driver.probe(device) {
					crate::info!(
						"Driver bound to existing device {}",
						device.name
					);
				}
			}
		}

		self.drivers.push(driver);
	}

	pub fn get_device(&self, id: DeviceId) -> Option<&AdvancedDevice> {
		self.devices.get(&id)
	}

	pub fn get_device_mut(&mut self, id: DeviceId) -> Option<&mut AdvancedDevice> {
		self.devices.get_mut(&id)
	}

	pub fn find_devices_by_class(&self, class: DeviceClass) -> Vec<DeviceId> {
		self.device_classes.get(&class).cloned().unwrap_or_default()
	}

	pub fn find_devices_by_name(&self, name: &str) -> Vec<DeviceId> {
		self.devices
			.iter()
			.filter(|(_, device)| device.name == name)
			.map(|(&id, _)| id)
			.collect()
	}

	pub fn get_device_statistics(&self) -> BTreeMap<DeviceClass, usize> {
		let mut stats = BTreeMap::new();
		for device in self.devices.values() {
			*stats.entry(device.class).or_insert(0) += 1;
		}
		stats
	}
}

/// Global advanced device registry
pub static ADVANCED_DEVICE_REGISTRY: Spinlock<AdvancedDeviceRegistry> =
	Spinlock::new(AdvancedDeviceRegistry::new());

/// Initialize advanced device management
pub fn init_advanced() -> Result<()> {
	crate::info!("Advanced device management initialized");
	Ok(())
}

/// Register a new advanced device
pub fn register_advanced_device(device: AdvancedDevice) -> Result<DeviceId> {
	let mut registry = ADVANCED_DEVICE_REGISTRY.lock();
	registry.register_device(device)
}

/// Register a device driver
pub fn register_device_driver(driver: Box<dyn AdvancedDeviceDriver>) {
	let mut registry = ADVANCED_DEVICE_REGISTRY.lock();
	registry.register_driver(driver);
}

/// Find devices by class
pub fn find_devices_by_class(class: DeviceClass) -> Vec<DeviceId> {
	let registry = ADVANCED_DEVICE_REGISTRY.lock();
	registry.find_devices_by_class(class)
}

/// Get device statistics
pub fn get_device_statistics() -> BTreeMap<DeviceClass, usize> {
	let registry = ADVANCED_DEVICE_REGISTRY.lock();
	registry.get_device_statistics()
}
