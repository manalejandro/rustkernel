// SPDX-License-Identifier: GPL-2.0

//! Null, zero, and full device drivers
//! Based on Linux drivers/char/mem.c

#![no_std]
#![no_main]

use kernel::device::{CharDevice, File, FileOperations, Inode, VMA};
use kernel::driver::CharDriverOps;
use kernel::prelude::*;

/// Null device driver (/dev/null)
#[derive(Debug)]
struct NullDevice;

impl FileOperations for NullDevice {
	fn open(&self, _inode: &Inode, _file: &mut File) -> Result<()> {
		Ok(())
	}

	fn release(&self, _inode: &Inode, _file: &mut File) -> Result<()> {
		Ok(())
	}

	fn read(&self, _file: &mut File, _buf: &mut [u8], _offset: u64) -> Result<usize> {
		// Reading from /dev/null always returns EOF
		Ok(0)
	}

	fn write(&self, _file: &mut File, buf: &[u8], _offset: u64) -> Result<usize> {
		// Writing to /dev/null always succeeds and discards data
		Ok(buf.len())
	}

	fn ioctl(&self, _file: &mut File, _cmd: u32, _arg: usize) -> Result<usize> {
		Err(Error::NotSupported)
	}

	fn mmap(&self, _file: &mut File, _vma: &mut VMA) -> Result<()> {
		Err(Error::NotSupported)
	}
}

/// Zero device driver (/dev/zero)
#[derive(Debug)]
struct ZeroDevice;

impl FileOperations for ZeroDevice {
	fn open(&self, _inode: &Inode, _file: &mut File) -> Result<()> {
		Ok(())
	}

	fn release(&self, _inode: &Inode, _file: &mut File) -> Result<()> {
		Ok(())
	}

	fn read(&self, _file: &mut File, buf: &mut [u8], _offset: u64) -> Result<usize> {
		// Reading from /dev/zero returns zeros
		for byte in buf.iter_mut() {
			*byte = 0;
		}
		Ok(buf.len())
	}

	fn write(&self, _file: &mut File, buf: &[u8], _offset: u64) -> Result<usize> {
		// Writing to /dev/zero always succeeds and discards data
		Ok(buf.len())
	}

	fn ioctl(&self, _file: &mut File, _cmd: u32, _arg: usize) -> Result<usize> {
		Err(Error::NotSupported)
	}

	fn mmap(&self, _file: &mut File, vma: &mut VMA) -> Result<()> {
		// /dev/zero can be mmap'd to get zero-filled pages
		// Implement proper mmap support for zero device
		crate::info!(
			"Mapping zero-filled pages at 0x{:x}",
			vma.vm_start.as_usize()
		);

		// In a real implementation, this would:
		// 1. Set up anonymous pages filled with zeros
		// 2. Configure page fault handler to provide zero pages on demand
		// 3. Mark pages as copy-on-write if needed

		// For now, just log the operation
		Ok(())
	}
}

/// Full device driver (/dev/full)
#[derive(Debug)]
struct FullDevice;

impl FileOperations for FullDevice {
	fn open(&self, _inode: &Inode, _file: &mut File) -> Result<()> {
		Ok(())
	}

	fn release(&self, _inode: &Inode, _file: &mut File) -> Result<()> {
		Ok(())
	}

	fn read(&self, _file: &mut File, buf: &mut [u8], _offset: u64) -> Result<usize> {
		// Reading from /dev/full returns zeros
		for byte in buf.iter_mut() {
			*byte = 0;
		}
		Ok(buf.len())
	}

	fn write(&self, _file: &mut File, _buf: &[u8], _offset: u64) -> Result<usize> {
		// Writing to /dev/full always fails with "no space left"
		Err(Error::OutOfMemory) // ENOSPC equivalent
	}

	fn ioctl(&self, _file: &mut File, _cmd: u32, _arg: usize) -> Result<usize> {
		Err(Error::NotSupported)
	}

	fn mmap(&self, _file: &mut File, _vma: &mut VMA) -> Result<()> {
		Err(Error::NotSupported)
	}
}

/// Memory devices module
struct MemoryDevicesModule {
	null_major: u32,
	zero_major: u32,
	full_major: u32,
}

impl kernel::module::Module for MemoryDevicesModule {
	fn init(_module: &'static kernel::module::ThisModule) -> Result<Self> {
		info!("Memory devices module initializing...");

		// Register /dev/null (major 1, minor 3)
		let null_major = kernel::device::register_chrdev(
			1,
			String::from("null"),
			Box::new(NullDevice),
		)?;

		// Register /dev/zero (major 1, minor 5)
		let zero_major = kernel::device::register_chrdev(
			1,
			String::from("zero"),
			Box::new(ZeroDevice),
		)?;

		// Register /dev/full (major 1, minor 7)
		let full_major = kernel::device::register_chrdev(
			1,
			String::from("full"),
			Box::new(FullDevice),
		)?;

		info!(
			"Memory devices registered: null={}, zero={}, full={}",
			null_major, zero_major, full_major
		);

		Ok(MemoryDevicesModule {
			null_major,
			zero_major,
			full_major,
		})
	}

	fn exit(_module: &'static kernel::module::ThisModule) {
		info!("Memory devices module exiting");

		// Unregister character devices
		kernel::device::unregister_chrdev(1).ok();
	}
}

module! {
    type: MemoryDevicesModule,
    name: "mem_devices",
    author: "Rust Kernel Contributors",
    description: "Memory devices (/dev/null, /dev/zero, /dev/full)",
    license: "GPL-2.0",
}
