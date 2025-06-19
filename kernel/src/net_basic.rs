// SPDX-License-Identifier: GPL-2.0

//! Basic networking support - loopback interface

use alloc::{collections::VecDeque, vec::Vec};

use crate::error::Result;
use crate::sync::Spinlock;
use crate::{info, warn};

/// Network packet
#[derive(Debug, Clone)]
pub struct NetPacket {
	pub data: Vec<u8>,
	pub length: usize,
	pub protocol: u16,
}

impl NetPacket {
	pub fn new(data: Vec<u8>, protocol: u16) -> Self {
		let length = data.len();
		Self {
			data,
			length,
			protocol,
		}
	}
}

/// Network interface
#[derive(Debug)]
pub struct NetInterface {
	pub name: &'static str,
	pub mtu: usize,
	pub tx_queue: VecDeque<NetPacket>,
	pub rx_queue: VecDeque<NetPacket>,
	pub stats: NetStats,
}

/// Network statistics
#[derive(Debug, Default)]
pub struct NetStats {
	pub tx_packets: u64,
	pub rx_packets: u64,
	pub tx_bytes: u64,
	pub rx_bytes: u64,
	pub tx_errors: u64,
	pub rx_errors: u64,
}

impl NetInterface {
	pub fn new(name: &'static str, mtu: usize) -> Self {
		Self {
			name,
			mtu,
			tx_queue: VecDeque::new(),
			rx_queue: VecDeque::new(),
			stats: NetStats::default(),
		}
	}

	/// Send a packet
	pub fn send_packet(&mut self, packet: NetPacket) -> Result<()> {
		if packet.length > self.mtu {
			self.stats.tx_errors += 1;
			return Err(crate::error::Error::InvalidArgument);
		}

		self.tx_queue.push_back(packet.clone());
		self.stats.tx_packets += 1;
		self.stats.tx_bytes += packet.length as u64;

		// For loopback, immediately receive the packet
		if self.name == "lo" {
			let rx_length = packet.length;
			self.rx_queue.push_back(packet);
			self.stats.rx_packets += 1;
			self.stats.rx_bytes += rx_length as u64;
		}

		Ok(())
	}

	/// Receive a packet
	pub fn receive_packet(&mut self) -> Option<NetPacket> {
		self.rx_queue.pop_front()
	}
}

/// Network subsystem
static NETWORK: Spinlock<NetworkSubsystem> = Spinlock::new(NetworkSubsystem::new());

struct NetworkSubsystem {
	interfaces: Vec<NetInterface>,
}

impl NetworkSubsystem {
	const fn new() -> Self {
		Self {
			interfaces: Vec::new(),
		}
	}

	fn add_interface(&mut self, interface: NetInterface) {
		info!("Adding network interface: {}", interface.name);
		self.interfaces.push(interface);
	}

	fn get_interface_mut(&mut self, name: &str) -> Option<&mut NetInterface> {
		self.interfaces.iter_mut().find(|iface| iface.name == name)
	}

	fn get_interface(&self, name: &str) -> Option<&NetInterface> {
		self.interfaces.iter().find(|iface| iface.name == name)
	}
}

/// Initialize basic networking
pub fn init_networking() -> Result<()> {
	info!("Initializing network subsystem");

	let mut network = NETWORK.lock();

	// Create loopback interface
	let loopback = NetInterface::new("lo", 65536);
	network.add_interface(loopback);

	info!("Network subsystem initialized");
	Ok(())
}

/// Send a packet on an interface
pub fn net_send(interface_name: &str, data: Vec<u8>, protocol: u16) -> Result<()> {
	let packet = NetPacket::new(data, protocol);
	let mut network = NETWORK.lock();

	if let Some(interface) = network.get_interface_mut(interface_name) {
		interface.send_packet(packet)?;
		info!("Sent packet on interface {}", interface_name);
		Ok(())
	} else {
		warn!("Network interface not found: {}", interface_name);
		Err(crate::error::Error::NotFound)
	}
}

/// Receive a packet from an interface
pub fn net_receive(interface_name: &str) -> Option<NetPacket> {
	let mut network = NETWORK.lock();

	if let Some(interface) = network.get_interface_mut(interface_name) {
		interface.receive_packet()
	} else {
		None
	}
}

/// Get network statistics
pub fn get_net_stats(interface_name: &str) -> Option<NetStats> {
	let network = NETWORK.lock();

	if let Some(interface) = network.get_interface(interface_name) {
		Some(NetStats {
			tx_packets: interface.stats.tx_packets,
			rx_packets: interface.stats.rx_packets,
			tx_bytes: interface.stats.tx_bytes,
			rx_bytes: interface.stats.rx_bytes,
			tx_errors: interface.stats.tx_errors,
			rx_errors: interface.stats.rx_errors,
		})
	} else {
		None
	}
}

/// Test networking functionality
pub fn test_networking() -> Result<()> {
	info!("Testing network functionality");

	// Test loopback
	let test_data = b"Hello, network!".to_vec();
	net_send("lo", test_data.clone(), 0x0800)?; // IP protocol

	if let Some(packet) = net_receive("lo") {
		if packet.data == test_data {
			info!("Loopback test passed");
		} else {
			warn!("Loopback test failed - data mismatch");
			return Err(crate::error::Error::Generic);
		}
	} else {
		warn!("Loopback test failed - no packet received");
		return Err(crate::error::Error::Generic);
	}

	// Display statistics
	if let Some(stats) = get_net_stats("lo") {
		info!(
			"Loopback stats: TX: {} packets/{} bytes, RX: {} packets/{} bytes",
			stats.tx_packets, stats.tx_bytes, stats.rx_packets, stats.rx_bytes
		);
	}

	Ok(())
}
