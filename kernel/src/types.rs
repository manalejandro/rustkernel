// SPDX-License-Identifier: GPL-2.0

//! Common kernel types

use core::fmt;
use core::ops::{Add, Mul, Sub};

/// Process ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(pub u32);

impl fmt::Display for Pid {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// Thread ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tid(pub u32);

impl fmt::Display for Tid {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

/// User ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uid(pub u32);

/// Group ID type  
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gid(pub u32);

/// Physical address type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

impl PhysAddr {
	pub const fn new(addr: usize) -> Self {
		Self(addr)
	}

	pub const fn as_u64(self) -> u64 {
		self.0 as u64
	}

	pub const fn as_usize(self) -> usize {
		self.0
	}
}

impl Add<usize> for PhysAddr {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		Self(self.0 + rhs)
	}
}

impl Sub<usize> for PhysAddr {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		Self(self.0 - rhs)
	}
}

/// Virtual address type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

impl VirtAddr {
	pub const fn new(addr: usize) -> Self {
		Self(addr)
	}

	pub const fn as_u64(self) -> u64 {
		self.0 as u64
	}

	pub const fn as_usize(self) -> usize {
		self.0
	}

	pub const fn as_ptr<T>(self) -> *const T {
		self.0 as *const T
	}

	pub const fn as_mut_ptr<T>(self) -> *mut T {
		self.0 as *mut T
	}
}

impl Add<usize> for VirtAddr {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		Self(self.0 + rhs)
	}
}

impl Sub<usize> for VirtAddr {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		Self(self.0 - rhs)
	}
}

impl Sub<VirtAddr> for VirtAddr {
	type Output = usize;

	fn sub(self, rhs: VirtAddr) -> Self::Output {
		self.0 - rhs.0
	}
}

impl fmt::Display for VirtAddr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "0x{:x}", self.0)
	}
}

/// Page size constants
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

/// Page frame number
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pfn(pub usize);

impl Pfn {
	pub fn from_phys_addr(addr: PhysAddr) -> Self {
		Self(addr.0 >> PAGE_SHIFT)
	}

	pub fn to_phys_addr(self) -> PhysAddr {
		PhysAddr(self.0 << PAGE_SHIFT)
	}
}

/// CPU number type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuId(pub u32);

/// IRQ number type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Irq(pub u32);

/// Time types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Jiffies(pub u64);

impl Jiffies {
	pub fn as_u64(self) -> u64 {
		self.0
	}
}

impl Mul<u64> for Jiffies {
	type Output = u64;

	fn mul(self, rhs: u64) -> Self::Output {
		self.0 * rhs
	}
}

impl core::ops::Add<u64> for Jiffies {
	type Output = Jiffies;

	fn add(self, rhs: u64) -> Self::Output {
		Jiffies(self.0 + rhs)
	}
}

impl core::ops::Sub<Jiffies> for Jiffies {
	type Output = Jiffies;

	fn sub(self, rhs: Jiffies) -> Self::Output {
		Jiffies(self.0.saturating_sub(rhs.0))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nanoseconds(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Microseconds(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Milliseconds(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Seconds(pub u64);

/// Device ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(pub u32);

impl fmt::Display for DeviceId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}
