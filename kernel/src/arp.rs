// SPDX-License-Identifier: GPL-2.0

//! ARP (Address Resolution Protocol) implementation.

use crate::error::{Error, Result};
use crate::network::{Ipv4Address, MacAddress};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpOperation {
	Request = 1,
	Reply = 2,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ArpPacket {
	pub htype: [u8; 2],
	pub ptype: [u8; 2],
	pub hlen: u8,
	pub plen: u8,
	pub oper: [u8; 2],
	pub sha: MacAddress,
	pub spa: Ipv4Address,
	pub tha: MacAddress,
	pub tpa: Ipv4Address,
}

impl ArpPacket {
	pub fn new(
		oper: ArpOperation,
		sha: MacAddress,
		spa: Ipv4Address,
		tha: MacAddress,
		tpa: Ipv4Address,
	) -> Self {
		Self {
			htype: (1 as u16).to_be_bytes(),      // Ethernet
			ptype: (0x0800 as u16).to_be_bytes(), // IPv4
			hlen: 6,
			plen: 4,
			oper: (oper as u16).to_be_bytes(),
			sha,
			spa,
			tha,
			tpa,
		}
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(28);
		bytes.extend_from_slice(&self.htype);
		bytes.extend_from_slice(&self.ptype);
		bytes.push(self.hlen);
		bytes.push(self.plen);
		bytes.extend_from_slice(&self.oper);
		bytes.extend_from_slice(self.sha.bytes());
		bytes.extend_from_slice(self.spa.bytes());
		bytes.extend_from_slice(self.tha.bytes());
		bytes.extend_from_slice(self.tpa.bytes());
		bytes
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
		if bytes.len() < 28 {
			return Err(Error::InvalidArgument);
		}
		let mut htype = [0u8; 2];
		htype.copy_from_slice(&bytes[0..2]);
		let mut ptype = [0u8; 2];
		ptype.copy_from_slice(&bytes[2..4]);
		let hlen = bytes[4];
		let plen = bytes[5];
		let mut oper = [0u8; 2];
		oper.copy_from_slice(&bytes[6..8]);
		let mut sha_bytes = [0u8; 6];
		sha_bytes.copy_from_slice(&bytes[8..14]);
		let mut spa_bytes = [0u8; 4];
		spa_bytes.copy_from_slice(&bytes[14..18]);
		let mut tha_bytes = [0u8; 6];
		tha_bytes.copy_from_slice(&bytes[18..24]);
		let mut tpa_bytes = [0u8; 4];
		tpa_bytes.copy_from_slice(&bytes[24..28]);

		Ok(Self {
			htype,
			ptype,
			hlen,
			plen,
			oper,
			sha: MacAddress::new(sha_bytes),
			spa: Ipv4Address::from_bytes(spa_bytes),
			tha: MacAddress::new(tha_bytes),
			tpa: Ipv4Address::from_bytes(tpa_bytes),
		})
	}
}
