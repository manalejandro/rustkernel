// SPDX-License-Identifier: GPL-2.0

//! Advanced networking stack implementation

use crate::error::{Error, Result};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// Network interface statistics
#[derive(Debug, Clone, Default)]
pub struct NetStats {
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
}

/// Network interface configuration
#[derive(Debug, Clone)]
pub struct NetConfig {
    pub name: String,
    pub mac_address: [u8; 6],
    pub ip_address: [u8; 4],
    pub netmask: [u8; 4],
    pub gateway: [u8; 4],
    pub mtu: u16,
    pub flags: u32,
}

/// Network packet
#[derive(Debug, Clone)]
pub struct NetPacket {
    pub data: Vec<u8>,
    pub length: usize,
    pub interface: String,
    pub timestamp: u64,
}

/// Network interface
pub struct NetworkInterface {
    pub config: NetConfig,
    pub stats: Mutex<NetStats>,
    pub rx_queue: Mutex<Vec<NetPacket>>,
    pub tx_queue: Mutex<Vec<NetPacket>>,
}

impl NetworkInterface {
    pub fn new(config: NetConfig) -> Self {
        Self {
            config,
            stats: Mutex::new(NetStats::default()),
            rx_queue: Mutex::new(Vec::new()),
            tx_queue: Mutex::new(Vec::new()),
        }
    }
    
    /// Send a packet
    pub fn send_packet(&self, data: &[u8]) -> Result<()> {
        let packet = NetPacket {
            data: data.to_vec(),
            length: data.len(),
            interface: self.config.name.clone(),
            timestamp: crate::time::get_time_ns(),
        };
        
        let mut tx_queue = self.tx_queue.lock();
        tx_queue.push(packet);
        
        let mut stats = self.stats.lock();
        stats.tx_packets += 1;
        stats.tx_bytes += data.len() as u64;
        
        crate::info!("Packet sent on interface {}: {} bytes", self.config.name, data.len());
        Ok(())
    }
    
    /// Receive a packet
    pub fn receive_packet(&self) -> Option<NetPacket> {
        let mut rx_queue = self.rx_queue.lock();
        if let Some(packet) = rx_queue.pop() {
            let mut stats = self.stats.lock();
            stats.rx_packets += 1;
            stats.rx_bytes += packet.length as u64;
            Some(packet)
        } else {
            None
        }
    }
    
    /// Get interface statistics
    pub fn get_stats(&self) -> NetStats {
        self.stats.lock().clone()
    }
}

/// Network stack
pub struct NetworkStack {
    interfaces: Mutex<BTreeMap<String, NetworkInterface>>,
    routing_table: Mutex<Vec<Route>>,
    arp_table: Mutex<BTreeMap<[u8; 4], [u8; 6]>>,
}

/// Routing table entry
#[derive(Debug, Clone)]
pub struct Route {
    pub destination: [u8; 4],
    pub netmask: [u8; 4],
    pub gateway: [u8; 4],
    pub interface: String,
    pub metric: u32,
}

impl NetworkStack {
    pub fn new() -> Self {
        Self {
            interfaces: Mutex::new(BTreeMap::new()),
            routing_table: Mutex::new(Vec::new()),
            arp_table: Mutex::new(BTreeMap::new()),
        }
    }
    
    /// Add network interface
    pub fn add_interface(&self, interface: NetworkInterface) -> Result<()> {
        let name = interface.config.name.clone();
        let mut interfaces = self.interfaces.lock();
        interfaces.insert(name.clone(), interface);
        
        crate::info!("Network interface {} added", name);
        Ok(())
    }
    
    /// Remove network interface
    pub fn remove_interface(&self, name: &str) -> Result<()> {
        let mut interfaces = self.interfaces.lock();
        if interfaces.remove(name).is_some() {
            crate::info!("Network interface {} removed", name);
            Ok(())
        } else {
            Err(Error::ENODEV)
        }
    }
    
    /// Get interface by name
    pub fn get_interface(&self, name: &str) -> Option<NetworkInterface> {
        let interfaces = self.interfaces.lock();
        interfaces.get(name).cloned()
    }
    
    /// List all interfaces
    pub fn list_interfaces(&self) -> Vec<String> {
        let interfaces = self.interfaces.lock();
        interfaces.keys().cloned().collect()
    }
    
    /// Add route
    pub fn add_route(&self, route: Route) -> Result<()> {
        let mut routing_table = self.routing_table.lock();
        routing_table.push(route);
        
        crate::info!("Route added to routing table");
        Ok(())
    }
    
    /// Find route for destination
    pub fn find_route(&self, destination: [u8; 4]) -> Option<Route> {
        let routing_table = self.routing_table.lock();
        
        // Simple routing - find exact match first, then default route
        for route in routing_table.iter() {
            if Self::ip_matches(&destination, &route.destination, &route.netmask) {
                return Some(route.clone());
            }
        }
        
        // Look for default route (0.0.0.0/0)
        for route in routing_table.iter() {
            if route.destination == [0, 0, 0, 0] && route.netmask == [0, 0, 0, 0] {
                return Some(route.clone());
            }
        }
        
        None
    }
    
    /// Check if IP matches network
    fn ip_matches(ip: &[u8; 4], network: &[u8; 4], netmask: &[u8; 4]) -> bool {
        for i in 0..4 {
            if (ip[i] & netmask[i]) != (network[i] & netmask[i]) {
                return false;
            }
        }
        true
    }
    
    /// Add ARP entry
    pub fn add_arp_entry(&self, ip: [u8; 4], mac: [u8; 6]) -> Result<()> {
        let mut arp_table = self.arp_table.lock();
        arp_table.insert(ip, mac);
        
        crate::info!("ARP entry added: {:?} -> {:?}", ip, mac);
        Ok(())
    }
    
    /// Lookup MAC address for IP
    pub fn arp_lookup(&self, ip: [u8; 4]) -> Option<[u8; 6]> {
        let arp_table = self.arp_table.lock();
        arp_table.get(&ip).copied()
    }
    
    /// Send packet to destination
    pub fn send_to(&self, destination: [u8; 4], data: &[u8]) -> Result<()> {
        // Find route
        let route = self.find_route(destination)
            .ok_or(Error::EHOSTUNREACH)?;
        
        // Get interface
        let interfaces = self.interfaces.lock();
        let interface = interfaces.get(&route.interface)
            .ok_or(Error::ENODEV)?;
        
        // For now, just send on the interface
        interface.send_packet(data)?;
        
        Ok(())
    }
    
    /// Get network statistics
    pub fn get_network_stats(&self) -> Vec<(String, NetStats)> {
        let interfaces = self.interfaces.lock();
        interfaces.iter()
            .map(|(name, iface)| (name.clone(), iface.get_stats()))
            .collect()
    }
}

/// Global network stack instance
static NETWORK_STACK: Mutex<Option<NetworkStack>> = Mutex::new(None);

/// Initialize networking stack
pub fn init() -> Result<()> {
    let stack = NetworkStack::new();
    
    // Create loopback interface
    let loopback_config = NetConfig {
        name: "lo".to_string(),
        mac_address: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        ip_address: [127, 0, 0, 1],
        netmask: [255, 0, 0, 0],
        gateway: [0, 0, 0, 0],
        mtu: 65536,
        flags: 0x1, // IFF_UP
    };
    
    let loopback = NetworkInterface::new(loopback_config);
    stack.add_interface(loopback)?;
    
    // Add loopback route
    let loopback_route = Route {
        destination: [127, 0, 0, 0],
        netmask: [255, 0, 0, 0],
        gateway: [0, 0, 0, 0],
        interface: "lo".to_string(),
        metric: 0,
    };
    stack.add_route(loopback_route)?;
    
    *NETWORK_STACK.lock() = Some(stack);
    
    crate::info!("Advanced networking stack initialized");
    Ok(())
}

/// Get global network stack
pub fn get_network_stack() -> Result<&'static Mutex<Option<NetworkStack>>> {
    Ok(&NETWORK_STACK)
}

/// Network utility functions
pub mod utils {
    use super::*;
    
    /// Format IP address as string
    pub fn ip_to_string(ip: [u8; 4]) -> String {
        format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
    }
    
    /// Parse IP address from string
    pub fn string_to_ip(s: &str) -> Result<[u8; 4]> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return Err(Error::EINVAL);
        }
        
        let mut ip = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            ip[i] = part.parse().map_err(|_| Error::EINVAL)?;
        }
        
        Ok(ip)
    }
    
    /// Format MAC address as string
    pub fn mac_to_string(mac: [u8; 6]) -> String {
        format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5])
    }
    
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

/// Simple packet creation utilities
pub mod packet {
    use super::*;
    
    /// Create a simple test packet
    pub fn create_test_packet(size: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(size);
        for i in 0..size {
            data.push((i % 256) as u8);
        }
        data
    }
    
    /// Create ICMP ping packet
    pub fn create_ping_packet(id: u16, seq: u16, data: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        
        // ICMP header
        packet.push(8); // Type: Echo Request
        packet.push(0); // Code: 0
        packet.push(0); // Checksum (will be calculated)  
        packet.push(0);
        packet.extend_from_slice(&id.to_be_bytes());
        packet.extend_from_slice(&seq.to_be_bytes());
        
        // Data
        packet.extend_from_slice(data);
        
        // Calculate checksum
        let checksum = utils::calculate_checksum(&packet);
        packet[2] = (checksum >> 8) as u8;
        packet[3] = (checksum & 0xFF) as u8;
        
        packet
    }
}
