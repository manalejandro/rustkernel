// SPDX-License-Identifier: GPL-2.0

//! RTL8139 Network Driver

use alloc::boxed::Box;
use alloc::string::ToString;
use core::ptr;
use kernel::driver::{Driver, PciDevice, PciDeviceId, PciDriver};
use kernel::error::{Error, Result};
use kernel::memory::{allocator, vmalloc};
use kernel::network::NetworkInterface;
use kernel::pci_driver;
use kernel::types::PhysAddr;

const REG_MAC0: u16 = 0x00;
const REG_MAR0: u16 = 0x08;
const REG_RBSTART: u16 = 0x30;
const REG_CMD: u16 = 0x37;
const REG_IMR: u16 = 0x3C;
const REG_ISR: u16 = 0x3E;
const REG_RCR: u16 = 0x44;
const REG_CONFIG1: u16 = 0x52;

const REG_TX_STATUS_0: u16 = 0x10;
const REG_TX_START_0: u16 = 0x20;

const CMD_RESET: u8 = 0x10;
const CMD_RX_ENB: u8 = 0x08;
const CMD_TX_ENB: u8 = 0x04;

const IMR_ROK: u16 = 1 << 0;
const IMR_TOK: u16 = 1 << 2;

const RCR_AAP: u32 = 1 << 0; // AcceptAllPackets
const RCR_APM: u32 = 1 << 1; // AcceptPhysicalMatch
const RCR_AM: u32 = 1 << 2; // AcceptMulticast
const RCR_AB: u32 = 1 << 3; // AcceptBroadcast
const RCR_WRAP: u32 = 1 << 7;

#[derive(Debug)]
struct Rtl8139Device {
	mmio_base: usize,
	mac: [u8; 6],
	rx_buffer: *mut u8,
	tx_buffer: *mut u8,
	rx_buffer_pos: usize,
	tx_cur: usize,
	up: bool,
}

impl Rtl8139Device {
	fn new(mmio_base: usize) -> Self {
		Self {
			mmio_base,
			mac: [0; 6],
			rx_buffer: ptr::null_mut(),
			tx_buffer: ptr::null_mut(),
			rx_buffer_pos: 0,
			tx_cur: 0,
			up: false,
		}
	}

	fn read8(&self, offset: u16) -> u8 {
		unsafe { ptr::read_volatile((self.mmio_base + offset as usize) as *const u8) }
	}

	fn write8(&self, offset: u16, val: u8) {
		unsafe { ptr::write_volatile((self.mmio_base + offset as usize) as *mut u8, val) }
	}

	fn read16(&self, offset: u16) -> u16 {
		unsafe { ptr::read_volatile((self.mmio_base + offset as usize) as *const u16) }
	}

	fn write16(&self, offset: u16, val: u16) {
		unsafe { ptr::write_volatile((self.mmio_base + offset as usize) as *mut u16, val) }
	}

	fn read32(&self, offset: u16) -> u32 {
		unsafe { ptr::read_volatile((self.mmio_base + offset as usize) as *const u32) }
	}

	fn write32(&self, offset: u16, val: u32) {
		unsafe { ptr::write_volatile((self.mmio_base + offset as usize) as *mut u32, val) }
	}
}

#[derive(Debug)]
pub struct Rtl8139Driver;

impl NetworkInterface for Rtl8139Device {
	fn name(&self) -> &str {
		"eth0" // Or some other name
	}

	fn ip_address(&self) -> Option<kernel::network::Ipv4Address> {
		None // This will be set by the network stack
	}

	fn mac_address(&self) -> kernel::network::MacAddress {
		kernel::network::MacAddress::new(self.mac)
	}

	fn mtu(&self) -> u16 {
		1500
	}

	fn is_up(&self) -> bool {
		self.up
	}

	fn send_packet(&mut self, buffer: &kernel::network::NetworkBuffer) -> Result<()> {
		let tx_status_reg = REG_TX_STATUS_0 + (self.tx_cur * 4) as u16;

		// The transmit buffers are laid out contiguously in memory after the receive buffer.
		let tx_buffer = unsafe { self.tx_buffer.add(self.tx_cur * 2048) };

		// Copy the packet data to the transmit buffer.
		unsafe {
			ptr::copy_nonoverlapping(buffer.data().as_ptr(), tx_buffer, buffer.len());
		}

		// Write the buffer address to the transmit start register.
		let dma_addr = PhysAddr::new(tx_buffer as usize);
		self.write32(
			REG_TX_START_0 + (self.tx_cur * 4) as u16,
			dma_addr.as_usize() as u32,
		);

		// Write the packet size and flags to the transmit status register.
		self.write32(tx_status_reg, buffer.len() as u32);

		self.tx_cur = (self.tx_cur + 1) % 4;

		Ok(())
	}

	fn receive_packet(&mut self) -> Result<Option<kernel::network::NetworkBuffer>> {
		let isr = self.read16(REG_ISR);
		if (isr & IMR_ROK) == 0 {
			return Ok(None);
		}

		// Acknowledge the interrupt
		self.write16(REG_ISR, IMR_ROK);

		let rx_ptr = self.rx_buffer as *const u8;
		let _header = unsafe {
			ptr::read_unaligned(rx_ptr.add(self.rx_buffer_pos) as *const u16)
		};
		let len = unsafe {
			ptr::read_unaligned(rx_ptr.add(self.rx_buffer_pos + 2) as *const u16)
		};

		let data_ptr = unsafe { rx_ptr.add(self.rx_buffer_pos + 4) };
		let data = unsafe { core::slice::from_raw_parts(data_ptr, len as usize) };

		// The data includes the Ethernet header.
		if data.len() < 14 {
			return Err(Error::InvalidArgument);
		}

		let dest_mac = kernel::network::MacAddress::new([
			data[0], data[1], data[2], data[3], data[4], data[5],
		]);
		let src_mac = kernel::network::MacAddress::new([
			data[6], data[7], data[8], data[9], data[10], data[11],
		]);
		let ethertype = u16::from_be_bytes([data[12], data[13]]);

		let protocol = match ethertype {
			0x0800 => kernel::network::ProtocolType::IPv4,
			0x0806 => kernel::network::ProtocolType::ARP,
			_ => return Ok(None), // Unknown protocol
		};

		let mut buffer = kernel::network::NetworkBuffer::from_data(data[14..].to_vec());
		buffer.set_protocol(protocol);
		buffer.set_mac_addresses(src_mac, dest_mac);

		self.rx_buffer_pos = (self.rx_buffer_pos + len as usize + 4 + 3) & !3;
		if self.rx_buffer_pos > 8192 {
			self.rx_buffer_pos -= 8192;
		}
		self.write16(0x38, self.rx_buffer_pos as u16 - 16);

		Ok(Some(buffer))
	}

	fn set_up(&mut self, up: bool) -> Result<()> {
		if up {
			self.write8(REG_CMD, CMD_RX_ENB | CMD_TX_ENB);
		} else {
			self.write8(REG_CMD, 0x00);
		}
		self.up = up;
		Ok(())
	}

	fn set_mac_address(&mut self, _mac: kernel::network::MacAddress) -> Result<()> {
		// Not supported
		Err(Error::NotSupported)
	}
}

impl Driver for Rtl8139Driver {
	fn name(&self) -> &str {
		"rtl8139"
	}

	fn probe(&self, _device: &mut kernel::device::Device) -> Result<()> {
		// This will be called for a generic device.
		// We are a PCI driver, so we'll do our work in pci_probe.
		Ok(())
	}

	fn remove(&self, _device: &mut kernel::device::Device) -> Result<()> {
		Ok(())
	}
}

impl PciDriver for Rtl8139Driver {
	fn pci_ids(&self) -> &[PciDeviceId] {
		&[PciDeviceId::new(0x10EC, 0x8139)]
	}

	fn pci_probe(&self, pci_dev: &mut PciDevice) -> Result<()> {
		kernel::info!("Probing rtl8139 device");

		let bar0 = pci_dev.bars[0];
		if bar0.is_io() {
			return Err(Error::NotSupported);
		}

		let mmio_base = bar0.address;
		kernel::info!("RTL8139 MMIO base: {:#x}", mmio_base);

		let mmio_virt = vmalloc::vmap_phys(PhysAddr::new(mmio_base as usize), 0x100)?;
		kernel::info!("RTL8139 MMIO mapped to: {:#x}", mmio_virt.as_usize());

		let mut device = Rtl8139Device::new(mmio_virt.as_usize());

		// Power on
		device.write8(REG_CONFIG1, 0x00);

		// Reset
		device.write8(REG_CMD, CMD_RESET);
		while (device.read8(REG_CMD) & CMD_RESET) != 0 {}

		// Read MAC address
		for i in 0..6 {
			device.mac[i] = device.read8(REG_MAC0 + i as u16);
		}
		kernel::info!(
			"RTL8139 MAC address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
			device.mac[0],
			device.mac[1],
			device.mac[2],
			device.mac[3],
			device.mac[4],
			device.mac[5]
		);

		// Allocate DMA buffers
		let dma_pfn = allocator::alloc_pages(2, allocator::GfpFlags::DMA)?;
		let dma_addr = dma_pfn.to_phys_addr();
		device.rx_buffer = dma_addr.as_usize() as *mut u8;
		device.tx_buffer = (dma_addr.as_usize() + 8192) as *mut u8;

		// Initialize receive buffer
		device.write32(REG_RBSTART, dma_addr.as_usize() as u32);

		// Initialize transmit buffers
		for i in 0..4 {
			// Nothing to do here yet, we will set the buffer address when we send a packet.
		}

		// Enable RX and TX
		device.write8(REG_CMD, CMD_RX_ENB | CMD_TX_ENB);

		// Set RCR
		device.write32(REG_RCR, RCR_AAP | RCR_APM | RCR_AM | RCR_AB | RCR_WRAP);

		// Enable interrupts
		device.write16(REG_IMR, IMR_TOK | IMR_ROK);

		kernel::info!("RTL8139 device initialized");

		let mut boxed_device = Box::new(device);
		boxed_device.set_up(true)?;
		kernel::network::add_network_interface("eth0".to_string(), boxed_device)?;

		Ok(())
	}

	fn pci_remove(&self, _pci_dev: &mut PciDevice) -> Result<()> {
		Ok(())
	}
}

pci_driver!(Rtl8139Driver);
