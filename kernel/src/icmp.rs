// SPDX-License-Identifier: GPL-2.0

//! ICMP (Internet Control Message Protocol) implementation.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IcmpType {
	EchoReply = 0,
	EchoRequest = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IcmpCode {
	Echo = 0,
}

use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IcmpPacket {
	pub icmp_type: IcmpType,
	pub icmp_code: IcmpCode,
	pub checksum: u16,
	pub identifier: u16,
	pub sequence_number: u16,
}

impl IcmpPacket {
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::new();
		bytes.push(self.icmp_type as u8);
		bytes.push(self.icmp_code as u8);
		bytes.extend_from_slice(&self.checksum.to_be_bytes());
		bytes.extend_from_slice(&self.identifier.to_be_bytes());
		bytes.extend_from_slice(&self.sequence_number.to_be_bytes());
		bytes
	}
}
