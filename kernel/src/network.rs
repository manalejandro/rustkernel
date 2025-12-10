// SPDX-License-Identifier: GPL-2.0

//! Network stack implementation

use alloc::{
	boxed::Box,
	collections::BTreeMap,
	collections::VecDeque,
	string::{String, ToString},
	vec::Vec,
};
use core::fmt;

use crate::error::{Error, Result};
use crate::sync::Spinlock;

/// Network protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolType {
	Ethernet = 0x0001,
	IPv4 = 0x0800,
	IPv6 = 0x86DD,
	ARP = 0x0806,
	TCP = 6,
	UDP = 17,
	ICMP = 2, // Different value to avoid conflict with Ethernet
	ICMPv6 = 58,
}

/// MAC address (6 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
	pub const fn new(bytes: [u8; 6]) -> Self {
		Self(bytes)
	}

	pub const fn broadcast() -> Self {
		Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
	}

	pub const fn zero() -> Self {
		Self([0, 0, 0, 0, 0, 0])
	}

	pub fn bytes(&self) -> &[u8; 6] {
		&self.0
	}

	pub fn is_broadcast(&self) -> bool {
		*self == Self::broadcast()
	}

	pub fn is_multicast(&self) -> bool {
		(self.0[0] & 0x01) != 0
	}
}

impl fmt::Display for MacAddress {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
			self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
		)
	}
}

/// IPv4 address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
	pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
		Self([a, b, c, d])
	}

	pub const fn from_bytes(bytes: [u8; 4]) -> Self {
		Self(bytes)
	}

	pub const fn localhost() -> Self {
		Self([127, 0, 0, 1])
	}

	pub const fn broadcast() -> Self {
		Self([255, 255, 255, 255])
	}

	pub const fn any() -> Self {
		Self([0, 0, 0, 0])
	}

	pub fn bytes(&self) -> &[u8; 4] {
		&self.0
	}

	pub fn to_u32(&self) -> u32 {
		u32::from_be_bytes(self.0)
	}

	pub fn from_u32(addr: u32) -> Self {
		Self(addr.to_be_bytes())
	}

	pub fn is_private(&self) -> bool {
		matches!(self.0, [10, ..] | [172, 16..=31, ..] | [192, 168, ..])
	}

	pub fn is_multicast(&self) -> bool {
		(self.0[0] & 0xF0) == 0xE0
	}

	pub fn is_broadcast(&self) -> bool {
		*self == Self::broadcast()
	}
}

impl fmt::Display for Ipv4Address {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
	}
}

/// Network packet buffer
#[derive(Debug, Clone)]
pub struct NetworkBuffer {
	data: Vec<u8>,
	len: usize,
	protocol: ProtocolType,
	source_mac: Option<MacAddress>,
	dest_mac: Option<MacAddress>,
	source_ip: Option<Ipv4Address>,
	dest_ip: Option<Ipv4Address>,
	source_port: Option<u16>,
	dest_port: Option<u16>,
}

impl NetworkBuffer {
	pub fn new(capacity: usize) -> Self {
		Self {
			data: Vec::with_capacity(capacity),
			len: 0,
			protocol: ProtocolType::Ethernet,
			source_mac: None,
			dest_mac: None,
			source_ip: None,
			dest_ip: None,
			source_port: None,
			dest_port: None,
		}
	}

	pub fn from_data(data: Vec<u8>) -> Self {
		let len = data.len();
		Self {
			data,
			len,
			protocol: ProtocolType::Ethernet,
			source_mac: None,
			dest_mac: None,
			source_ip: None,
			dest_ip: None,
			source_port: None,
			dest_port: None,
		}
	}

	pub fn data(&self) -> &[u8] {
		&self.data[..self.len]
	}

	pub fn data_mut(&mut self) -> &mut [u8] {
		&mut self.data[..self.len]
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn push(&mut self, byte: u8) -> Result<()> {
		if self.len >= self.data.capacity() {
			return Err(Error::OutOfMemory);
		}
		if self.len >= self.data.len() {
			self.data.push(byte);
		} else {
			self.data[self.len] = byte;
		}
		self.len += 1;
		Ok(())
	}

	pub fn extend_from_slice(&mut self, data: &[u8]) -> Result<()> {
		if self.len + data.len() > self.data.capacity() {
			return Err(Error::OutOfMemory);
		}
		for &byte in data {
			self.push(byte)?;
		}
		Ok(())
	}

	pub fn set_protocol(&mut self, protocol: ProtocolType) {
		self.protocol = protocol;
	}

	pub fn set_mac_addresses(&mut self, source: MacAddress, dest: MacAddress) {
		self.source_mac = Some(source);
		self.dest_mac = Some(dest);
	}

	pub fn set_ip_addresses(&mut self, source: Ipv4Address, dest: Ipv4Address) {
		self.source_ip = Some(source);
		self.dest_ip = Some(dest);
	}

	pub fn set_ports(&mut self, source: u16, dest: u16) {
		self.source_port = Some(source);
		self.dest_port = Some(dest);
	}
}

/// Network interface
pub trait NetworkInterface: Send + Sync {
	fn name(&self) -> &str;
	fn ip_address(&self) -> Option<Ipv4Address>;
	fn mac_address(&self) -> MacAddress;
	fn mtu(&self) -> u16;
	fn is_up(&self) -> bool;

	fn send_packet(&mut self, buffer: &NetworkBuffer) -> Result<()>;
	fn receive_packet(&mut self) -> Result<Option<NetworkBuffer>>;

	fn set_up(&mut self, up: bool) -> Result<()>;
	fn set_mac_address(&mut self, mac: MacAddress) -> Result<()>;
}

/// Network interface statistics
#[derive(Debug, Default, Clone)]
pub struct InterfaceStats {
	pub bytes_sent: u64,
	pub bytes_received: u64,
	pub packets_sent: u64,
	pub packets_received: u64,
	pub errors: u64,
	pub dropped: u64,
}

/// A loopback network interface.
#[derive(Debug)]
pub struct LoopbackInterface {
	rx_queue: VecDeque<NetworkBuffer>,
	up: bool,
}

impl LoopbackInterface {
	pub fn new() -> Self {
		Self {
			rx_queue: VecDeque::new(),
			up: true,
		}
	}
}

impl NetworkInterface for LoopbackInterface {
	fn name(&self) -> &str {
		"lo"
	}

	fn ip_address(&self) -> Option<Ipv4Address> {
		Some(Ipv4Address::localhost())
	}

	fn mac_address(&self) -> MacAddress {
		MacAddress::zero()
	}

	fn mtu(&self) -> u16 {
		65535
	}

	fn is_up(&self) -> bool {
		self.up
	}

	fn send_packet(&mut self, buffer: &NetworkBuffer) -> Result<()> {
		if !self.up {
			return Err(Error::NetworkDown);
		}
		self.rx_queue.push_back(buffer.clone());
		Ok(())
	}

	fn receive_packet(&mut self) -> Result<Option<NetworkBuffer>> {
		if !self.up {
			return Ok(None);
		}
		Ok(self.rx_queue.pop_front())
	}

	fn set_up(&mut self, up: bool) -> Result<()> {
		self.up = up;
		Ok(())
	}

	fn set_mac_address(&mut self, _mac: MacAddress) -> Result<()> {
		// The loopback interface doesn't have a real MAC address.
		Ok(())
	}
}

/// Network stack
struct PendingArpRequest {
	packet: NetworkBuffer,
	ip: Ipv4Address,
	timestamp: u64,
}

pub struct NetworkStack {
	interfaces: BTreeMap<String, Box<dyn NetworkInterface>>,
	interface_stats: BTreeMap<String, InterfaceStats>,
	routing_table: Vec<RouteEntry>,
	arp_table: BTreeMap<Ipv4Address, MacAddress>,
	pending_arp_requests: Vec<PendingArpRequest>,
}

/// Routing table entry
#[derive(Debug, Clone)]
pub struct RouteEntry {
	pub destination: Ipv4Address,
	pub netmask: Ipv4Address,
	pub gateway: Option<Ipv4Address>,
	pub interface: String,
	pub metric: u32,
}

impl NetworkStack {
	const fn new() -> Self {
		Self {
			interfaces: BTreeMap::new(),
			interface_stats: BTreeMap::new(),
			routing_table: Vec::new(),
			arp_table: BTreeMap::new(),
			pending_arp_requests: Vec::new(),
		}
	}

	pub fn add_interface(&mut self, name: String, interface: Box<dyn NetworkInterface>) {
		self.interface_stats
			.insert(name.clone(), InterfaceStats::default());
		self.interfaces.insert(name, interface);
	}

	pub fn remove_interface(&mut self, name: &str) -> Option<Box<dyn NetworkInterface>> {
		self.interface_stats.remove(name);
		self.interfaces.remove(name)
	}

	pub fn get_interface(&self, name: &str) -> Option<&dyn NetworkInterface> {
		self.interfaces.get(name).map(|i| i.as_ref())
	}

	pub fn get_interface_mut<'a>(
		&'a mut self,
		name: &str,
	) -> Option<&'a mut (dyn NetworkInterface + 'a)> {
		if let Some(interface) = self.interfaces.get_mut(name) {
			Some(interface.as_mut())
		} else {
			None
		}
	}

	pub fn list_interfaces(&self) -> Vec<String> {
		self.interfaces.keys().cloned().collect()
	}

	pub fn add_route(&mut self, route: RouteEntry) {
		self.routing_table.push(route);
		// Sort by metric (lower is better)
		self.routing_table.sort_by_key(|r| r.metric);
	}

	pub fn find_route(&self, dest: Ipv4Address) -> Option<&RouteEntry> {
		for route in &self.routing_table {
			let dest_u32 = dest.to_u32();
			let route_dest = route.destination.to_u32();
			let netmask = route.netmask.to_u32();

			if (dest_u32 & netmask) == (route_dest & netmask) {
				return Some(route);
			}
		}
		None
	}

	pub fn add_arp_entry(&mut self, ip: Ipv4Address, mac: MacAddress) {
		self.arp_table.insert(ip, mac);
	}

	pub fn lookup_arp(&self, ip: Ipv4Address) -> Option<MacAddress> {
		self.arp_table.get(&ip).copied()
	}

	pub fn send_packet(
		&mut self,
		dest: Ipv4Address,
		data: &[u8],
		protocol: ProtocolType,
	) -> Result<()> {
		// Clean up timed out ARP requests
		let now = crate::time::get_time_ns();
		self.pending_arp_requests
			.retain(|req| now - req.timestamp < 10_000_000_000); // 10 seconds

		// Find route (borrow self immutably)
		let route = {
			let route = self.find_route(dest).ok_or(Error::NetworkUnreachable)?;
			route.clone() // Clone to avoid borrowing issues
		};

		// Look up MAC address first (borrow self immutably)
		let dest_mac = if let Some(gateway) = route.gateway {
			self.lookup_arp(gateway)
		} else {
			self.lookup_arp(dest)
		};

		let dest_mac = if let Some(mac) = dest_mac {
			mac
		} else {
			// ARP lookup failed, send an ARP request and queue the packet
			let interface = self
				.get_interface(&route.interface)
				.ok_or(Error::DeviceNotFound)?;
			let arp_request = crate::arp::ArpPacket::new(
				crate::arp::ArpOperation::Request,
				interface.mac_address(),
				interface.ip_address().unwrap_or(Ipv4Address::any()),
				MacAddress::zero(),
				dest,
			);
			let mut buffer = NetworkBuffer::new(28);
			buffer.set_protocol(ProtocolType::ARP);
			buffer.set_mac_addresses(interface.mac_address(), MacAddress::broadcast());
			buffer.extend_from_slice(&arp_request.to_bytes())?;
			let interface_mut = self
				.get_interface_mut(&route.interface)
				.ok_or(Error::DeviceNotFound)?;
			interface_mut.send_packet(&buffer)?;

			// Queue the original packet
			let mut packet_to_queue = NetworkBuffer::new(data.len());
			packet_to_queue.extend_from_slice(data)?;
			packet_to_queue.set_protocol(protocol);
			packet_to_queue.set_ip_addresses(Ipv4Address::any(), dest); // TODO: Set source IP
			self.pending_arp_requests.push(PendingArpRequest {
				packet: packet_to_queue,
				ip: dest,
				timestamp: crate::time::get_time_ns(),
			});

			return Ok(()); // We'll have to wait for the reply
		};

		// Get interface MAC address
		let interface_mac = {
			let interface = self
				.get_interface(&route.interface)
				.ok_or(Error::DeviceNotFound)?;
			interface.mac_address()
		};

		// Build packet
		let mut buffer = NetworkBuffer::new(1500);
		buffer.set_protocol(protocol);
		buffer.set_mac_addresses(interface_mac, dest_mac);
		buffer.extend_from_slice(data)?;

		// Send packet (borrow self mutably)
		{
			let interface = self
				.get_interface_mut(&route.interface)
				.ok_or(Error::DeviceNotFound)?;
			interface.send_packet(&buffer)?;
		}

		// Update statistics
		if let Some(stats) = self.interface_stats.get_mut(&route.interface) {
			stats.packets_sent += 1;
			stats.bytes_sent += buffer.len() as u64;
		}

		Ok(())
	}

	pub fn receive_and_handle_packets(&mut self) -> Result<Vec<NetworkBuffer>> {
		let mut received_packets = Vec::new();
		let mut unhandled_packets = Vec::new();

		// First, receive all packets from all interfaces
		for (name, interface) in &mut self.interfaces {
			while let Some(packet) = interface.receive_packet()? {
				if let Some(stats) = self.interface_stats.get_mut(name) {
					stats.packets_received += 1;
					stats.bytes_received += packet.len() as u64;
				}
				received_packets.push((name.clone(), packet));
			}
		}

		// Now, process the received packets
		for (interface_name, packet) in received_packets {
			if packet.protocol == ProtocolType::ARP {
				if let Ok(arp_packet) =
					crate::arp::ArpPacket::from_bytes(packet.data())
				{
					self.handle_arp_packet(&arp_packet, &interface_name)?;
				}
			} else if packet.protocol == ProtocolType::ICMP {
				if let Some(source_ip) = packet.source_ip {
					self.handle_icmp_packet(source_ip, packet.data())?;
				}
			} else {
				unhandled_packets.push(packet);
			}
		}

		Ok(unhandled_packets)
	}

	pub fn receive_packets(&mut self) -> Result<Vec<NetworkBuffer>> {
		self.receive_and_handle_packets()
	}

	pub fn get_interface_stats(&self, name: &str) -> Option<&InterfaceStats> {
		self.interface_stats.get(name)
	}

	fn handle_arp_packet(
		&mut self,
		packet: &crate::arp::ArpPacket,
		interface_name: &str,
	) -> Result<()> {
		// Add the sender to the ARP table
		self.add_arp_entry(packet.spa, packet.sha);

		// If it's a request for us, send a reply
		if u16::from_be_bytes(packet.oper) == crate::arp::ArpOperation::Request as u16 {
			if let Some(interface) = self.get_interface(interface_name) {
				if let Some(ip_addr) = interface.ip_address() {
					if ip_addr == packet.tpa {
						let reply = crate::arp::ArpPacket::new(
							crate::arp::ArpOperation::Reply,
							interface.mac_address(),
							ip_addr,
							packet.sha,
							packet.spa,
						);
						let mut buffer = NetworkBuffer::new(28);
						buffer.set_protocol(ProtocolType::ARP);
						buffer.set_mac_addresses(
							interface.mac_address(),
							packet.sha,
						);
						buffer.extend_from_slice(&reply.to_bytes())?;
						if let Some(interface) =
							self.get_interface_mut(interface_name)
						{
							interface.send_packet(&buffer)?;
						}
					}
				}
			}
		}

		// Check for pending packets
		let mut packets_to_send = Vec::new();
		let mut still_pending = Vec::new();
		for pending in self.pending_arp_requests.drain(..) {
			if pending.ip == packet.spa {
				packets_to_send.push(pending);
			} else {
				still_pending.push(pending);
			}
		}
		self.pending_arp_requests = still_pending;

		for pending in packets_to_send {
			self.send_packet(
				pending.ip,
				pending.packet.data(),
				pending.packet.protocol,
			)?;
		}

		Ok(())
	}

	fn handle_icmp_packet(&mut self, source_ip: Ipv4Address, packet: &[u8]) -> Result<()> {
		if packet.len() < 8 {
			return Err(Error::InvalidArgument);
		}
		let icmp_type = packet[0];
		if icmp_type == crate::icmp::IcmpType::EchoRequest as u8 {
			let mut reply = packet.to_vec();
			reply[0] = crate::icmp::IcmpType::EchoReply as u8;
			// Recalculate checksum
			let checksum = utils::calculate_checksum(&reply);
			reply[2] = (checksum >> 8) as u8;
			reply[3] = (checksum & 0xFF) as u8;

			self.send_packet(source_ip, &reply, ProtocolType::ICMP)?;
		}
		Ok(())
	}
}

/// Global network stack
pub static NETWORK_STACK: Spinlock<Option<NetworkStack>> = Spinlock::new(None);

/// Initialize network stack
pub fn init() -> Result<()> {
	let mut stack = NETWORK_STACK.lock();
	let mut network_stack = NetworkStack::new();

	// Add loopback interface
	let loopback = LoopbackInterface::new();
	network_stack.add_interface("lo".to_string(), Box::new(loopback));

	// Add route for loopback
	network_stack.add_route(RouteEntry {
		destination: Ipv4Address::new(127, 0, 0, 0),
		netmask: Ipv4Address::new(255, 0, 0, 0),
		gateway: None,
		interface: "lo".to_string(),
		metric: 0,
	});

	// Add ARP entry for loopback
	network_stack.add_arp_entry(Ipv4Address::localhost(), MacAddress::zero());

	*stack = Some(network_stack);
	crate::info!("Network stack initialized");
	Ok(())
}

/// Add a network interface
pub fn add_network_interface(name: String, interface: Box<dyn NetworkInterface>) -> Result<()> {
	let mut stack_opt = NETWORK_STACK.lock();
	if let Some(ref mut stack) = *stack_opt {
		stack.add_interface(name, interface);
		Ok(())
	} else {
		Err(Error::NotInitialized)
	}
}

pub mod utils {
	/// Calculate checksum
	pub fn calculate_checksum(data: &[u8]) -> u16 {
		let mut sum = 0u32;

		// Sum all 16-bit words
		for chunk in data.chunks(2) {
			if chunk.len() == 2 {
				sum += ((chunk[0] as u32) << 8) + (chunk[1] as u32);
			} else {
				sum += (chunk[0] as u32) << 8;
			}
		}

		// Add carry
		while sum >> 16 != 0 {
			sum = (sum & 0xFFFF) + (sum >> 16);
		}

		// One's complement
		!sum as u16
	}
}

/// Send a packet
pub fn send_packet(dest: Ipv4Address, data: &[u8], protocol: ProtocolType) -> Result<()> {
	let mut stack_opt = NETWORK_STACK.lock();
	if let Some(ref mut stack) = *stack_opt {
		stack.send_packet(dest, data, protocol)
	} else {
		Err(Error::NotInitialized)
	}
}

/// Add a route
pub fn add_route(
	destination: Ipv4Address,
	netmask: Ipv4Address,
	gateway: Option<Ipv4Address>,
	interface: String,
	metric: u32,
) -> Result<()> {
	let mut stack_opt = NETWORK_STACK.lock();
	if let Some(ref mut stack) = *stack_opt {
		stack.add_route(RouteEntry {
			destination,
			netmask,
			gateway,
			interface,
			metric,
		});
		Ok(())
	} else {
		Err(Error::NotInitialized)
	}
}

/// Add an ARP entry
pub fn add_arp_entry(ip: Ipv4Address, mac: MacAddress) -> Result<()> {
	let mut stack_opt = NETWORK_STACK.lock();
	if let Some(ref mut stack) = *stack_opt {
		stack.add_arp_entry(ip, mac);
		Ok(())
	} else {
		Err(Error::NotInitialized)
	}
}
