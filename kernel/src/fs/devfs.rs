// SPDX-License-Identifier: GPL-2.0

//! Character device filesystem integration
//!
//! This driver provides VFS integration for character devices
//! like /dev/null, /dev/zero, /dev/random, etc.

use alloc::{boxed::Box, string::String, vec};

use crate::error::{Error, Result};
use crate::fs::*;
use crate::memory::UserSlicePtr;
use crate::sync::Arc; // Import vec macro and Box

/// Character device file operations
#[derive(Debug)]
pub struct CharDevFileOps {
	/// Device operations
	dev_ops: Option<Arc<dyn CharDevOperations>>,
}

impl CharDevFileOps {
	pub fn new(dev_ops: Option<Arc<dyn CharDevOperations>>) -> Self {
		Self { dev_ops }
	}
}

impl FileOperations for CharDevFileOps {
	fn read(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		if let Some(ref ops) = self.dev_ops {
			ops.read(file, buf, count)
		} else {
			Err(Error::ENODEV)
		}
	}

	fn write(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		if let Some(ref ops) = self.dev_ops {
			ops.write(file, buf, count)
		} else {
			Err(Error::ENODEV)
		}
	}

	fn seek(&self, file: &File, offset: i64, whence: i32) -> Result<i64> {
		// Most character devices don't support seeking
		Err(Error::ESPIPE)
	}

	fn ioctl(&self, file: &File, cmd: u32, arg: usize) -> Result<isize> {
		if let Some(ref ops) = self.dev_ops {
			ops.ioctl(file, cmd, arg)
		} else {
			Err(Error::ENOTTY)
		}
	}

	fn mmap(&self, file: &File, vma: &mut crate::memory::VmaArea) -> Result<()> {
		if let Some(ref ops) = self.dev_ops {
			ops.mmap(file, vma)
		} else {
			Err(Error::ENODEV)
		}
	}

	fn fsync(&self, file: &File, datasync: bool) -> Result<()> {
		// Character devices don't need syncing
		Ok(())
	}

	fn poll(&self, file: &File, wait: &mut PollWait) -> Result<u32> {
		if let Some(ref ops) = self.dev_ops {
			ops.poll(file, wait)
		} else {
			Ok(POLLIN | POLLOUT)
		}
	}

	fn open(&self, inode: &Inode, file: &File) -> Result<()> {
		if let Some(ref ops) = self.dev_ops {
			ops.open(inode, file)
		} else {
			Ok(())
		}
	}

	fn release(&self, inode: &Inode, file: &File) -> Result<()> {
		if let Some(ref ops) = self.dev_ops {
			ops.release(inode, file)
		} else {
			Ok(())
		}
	}
}

/// Character device operations trait
pub trait CharDevOperations: Send + Sync + core::fmt::Debug {
	fn read(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize>;
	fn write(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize>;
	fn ioctl(&self, file: &File, cmd: u32, arg: usize) -> Result<isize>;
	fn mmap(&self, file: &File, vma: &mut crate::memory::VmaArea) -> Result<()>;
	fn poll(&self, file: &File, wait: &mut PollWait) -> Result<u32>;
	fn open(&self, inode: &Inode, file: &File) -> Result<()>;
	fn release(&self, inode: &Inode, file: &File) -> Result<()>;
}

/// /dev/null device operations
#[derive(Debug)]
pub struct NullDevOps;

impl CharDevOperations for NullDevOps {
	fn read(&self, _file: &File, _buf: UserSlicePtr, _count: usize) -> Result<isize> {
		Ok(0) // EOF
	}

	fn write(&self, _file: &File, _buf: UserSlicePtr, count: usize) -> Result<isize> {
		Ok(count as isize) // Discard all data
	}

	fn ioctl(&self, _file: &File, _cmd: u32, _arg: usize) -> Result<isize> {
		Err(Error::ENOTTY)
	}

	fn mmap(&self, _file: &File, _vma: &mut crate::memory::VmaArea) -> Result<()> {
		Err(Error::ENODEV)
	}

	fn poll(&self, _file: &File, _wait: &mut PollWait) -> Result<u32> {
		Ok(POLLIN | POLLOUT) // Always ready
	}

	fn open(&self, _inode: &Inode, _file: &File) -> Result<()> {
		Ok(())
	}

	fn release(&self, _inode: &Inode, _file: &File) -> Result<()> {
		Ok(())
	}
}

/// /dev/zero device operations
#[derive(Debug)]
pub struct ZeroDevOps;

impl CharDevOperations for ZeroDevOps {
	fn read(&self, _file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Fill buffer with zeros
		let zeros = vec![0u8; count];
		buf.copy_from_slice(&zeros)?;
		Ok(count as isize)
	}

	fn write(&self, _file: &File, _buf: UserSlicePtr, count: usize) -> Result<isize> {
		Ok(count as isize) // Discard all data
	}

	fn ioctl(&self, _file: &File, _cmd: u32, _arg: usize) -> Result<isize> {
		Err(Error::ENOTTY)
	}

	fn mmap(&self, _file: &File, _vma: &mut crate::memory::VmaArea) -> Result<()> {
		// TODO: Map zero page
		Err(Error::ENODEV)
	}

	fn poll(&self, _file: &File, _wait: &mut PollWait) -> Result<u32> {
		Ok(POLLIN | POLLOUT) // Always ready
	}

	fn open(&self, _inode: &Inode, _file: &File) -> Result<()> {
		Ok(())
	}

	fn release(&self, _inode: &Inode, _file: &File) -> Result<()> {
		Ok(())
	}
}

/// /dev/full device operations
#[derive(Debug)]
pub struct FullDevOps;

impl CharDevOperations for FullDevOps {
	fn read(&self, _file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Fill buffer with zeros (like /dev/zero)
		let zeros = vec![0u8; count];
		buf.copy_from_slice(&zeros)?;
		Ok(count as isize)
	}

	fn write(&self, _file: &File, _buf: UserSlicePtr, _count: usize) -> Result<isize> {
		Err(Error::ENOSPC) // No space left on device
	}

	fn ioctl(&self, _file: &File, _cmd: u32, _arg: usize) -> Result<isize> {
		Err(Error::ENOTTY)
	}

	fn mmap(&self, _file: &File, _vma: &mut crate::memory::VmaArea) -> Result<()> {
		Err(Error::ENODEV)
	}

	fn poll(&self, _file: &File, _wait: &mut PollWait) -> Result<u32> {
		Ok(POLLIN) // Only readable
	}

	fn open(&self, _inode: &Inode, _file: &File) -> Result<()> {
		Ok(())
	}

	fn release(&self, _inode: &Inode, _file: &File) -> Result<()> {
		Ok(())
	}
}

/// /dev/random device operations (simplified)
#[derive(Debug)]
pub struct RandomDevOps;

impl CharDevOperations for RandomDevOps {
	fn read(&self, _file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Generate pseudo-random data
		// TODO: Use proper random number generator
		let mut random_data = vec![0u8; count];
		for i in 0..count {
			random_data[i] = (i * 37 + 13) as u8; // Very simple PRNG
		}
		buf.copy_from_slice(&random_data)?;
		Ok(count as isize)
	}

	fn write(&self, _file: &File, _buf: UserSlicePtr, count: usize) -> Result<isize> {
		// TODO: Add entropy to random pool
		Ok(count as isize)
	}

	fn ioctl(&self, _file: &File, _cmd: u32, _arg: usize) -> Result<isize> {
		// TODO: Implement random device ioctls
		Err(Error::ENOTTY)
	}

	fn mmap(&self, _file: &File, _vma: &mut crate::memory::VmaArea) -> Result<()> {
		Err(Error::ENODEV)
	}

	fn poll(&self, _file: &File, _wait: &mut PollWait) -> Result<u32> {
		Ok(POLLIN | POLLOUT) // Always ready
	}

	fn open(&self, _inode: &Inode, _file: &File) -> Result<()> {
		Ok(())
	}

	fn release(&self, _inode: &Inode, _file: &File) -> Result<()> {
		Ok(())
	}
}

/// Create a character device inode
pub fn create_char_device_inode(
	major: u32,
	minor: u32,
	mode: u32,
	ops: Arc<dyn CharDevOperations>,
) -> Arc<Inode> {
	let mut inode = Inode::new(0, mode::S_IFCHR | mode);
	inode.i_rdev = crate::device::DeviceNumber::new(major, minor);
	inode.set_file_operations(Arc::new(CharDevFileOps::new(Some(ops))));

	Arc::new(inode)
}

/// DevFS - device filesystem for /dev
pub struct DevFs {
	/// Device entries
	devices: crate::sync::Mutex<alloc::collections::BTreeMap<String, Arc<Inode>>>,
}

impl DevFs {
	pub fn new() -> Self {
		let mut devfs = Self {
			devices: crate::sync::Mutex::new(alloc::collections::BTreeMap::new()),
		};

		devfs.create_standard_devices();
		devfs
	}

	fn create_standard_devices(&mut self) {
		// Create /dev/null
		let null_inode = create_char_device_inode(1, 3, 0o666, Arc::new(NullDevOps));
		self.add_device("null", null_inode);

		// Create /dev/zero
		let zero_inode = create_char_device_inode(1, 5, 0o666, Arc::new(ZeroDevOps));
		self.add_device("zero", zero_inode);

		// Create /dev/full
		let full_inode = create_char_device_inode(1, 7, 0o666, Arc::new(FullDevOps));
		self.add_device("full", full_inode);

		// Create /dev/random
		let random_inode = create_char_device_inode(1, 8, 0o666, Arc::new(RandomDevOps));
		self.add_device("random", random_inode);

		// Create /dev/urandom (same as random for now)
		let urandom_inode = create_char_device_inode(1, 9, 0o666, Arc::new(RandomDevOps));
		self.add_device("urandom", urandom_inode);
	}

	pub fn add_device(&mut self, name: &str, inode: Arc<Inode>) {
		let mut devices = self.devices.lock();
		devices.insert(String::from(name), inode);
	}

	pub fn get_device(&self, name: &str) -> Option<Arc<Inode>> {
		let devices = self.devices.lock();
		devices.get(name).cloned()
	}

	pub fn list_devices(&self) -> alloc::vec::Vec<String> {
		let devices = self.devices.lock();
		devices.keys().cloned().collect()
	}
}

/// DevFS inode operations
#[derive(Debug)]
pub struct DevFsInodeOps {
	devfs: *const DevFs,
}

impl DevFsInodeOps {
	pub fn new(devfs: &DevFs) -> Self {
		Self {
			devfs: devfs as *const DevFs,
		}
	}

	fn get_devfs(&self) -> &DevFs {
		unsafe { &*self.devfs }
	}
}

unsafe impl Send for DevFsInodeOps {}
unsafe impl Sync for DevFsInodeOps {}

impl InodeOperations for DevFsInodeOps {
	fn lookup(&self, dir: &Inode, name: &str) -> Result<Arc<Inode>> {
		let devfs = self.get_devfs();
		devfs.get_device(name).ok_or(Error::ENOENT)
	}

	fn create(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>> {
		Err(Error::EPERM) // Can't create devices in /dev directly
	}

	fn mkdir(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>> {
		Err(Error::EPERM)
	}

	fn unlink(&self, dir: &Inode, name: &str) -> Result<()> {
		Err(Error::EPERM)
	}

	fn rmdir(&self, dir: &Inode, name: &str) -> Result<()> {
		Err(Error::EPERM)
	}

	fn symlink(&self, dir: &Inode, name: &str, target: &str) -> Result<Arc<Inode>> {
		Err(Error::EPERM)
	}

	fn rename(
		&self,
		old_dir: &Inode,
		old_name: &str,
		new_dir: &Inode,
		new_name: &str,
	) -> Result<()> {
		Err(Error::EPERM)
	}

	fn setattr(&self, inode: &Inode, attr: &InodeAttr) -> Result<()> {
		Err(Error::EPERM)
	}

	fn getattr(&self, inode: &Inode) -> Result<InodeAttr> {
		let generic_ops = GenericInodeOps;
		generic_ops.getattr(inode)
	}

	fn readlink(&self, inode: &Inode) -> Result<String> {
		Err(Error::EINVAL)
	}

	fn follow_link(&self, inode: &Inode) -> Result<Arc<Inode>> {
		Err(Error::EINVAL)
	}

	fn truncate(&self, inode: &Inode, size: u64) -> Result<()> {
		Err(Error::EPERM)
	}

	fn getxattr(&self, inode: &Inode, name: &str) -> Result<alloc::vec::Vec<u8>> {
		Err(Error::ENODATA)
	}

	fn setxattr(&self, inode: &Inode, name: &str, value: &[u8], flags: u32) -> Result<()> {
		Err(Error::EPERM)
	}

	fn listxattr(&self, inode: &Inode) -> Result<alloc::vec::Vec<String>> {
		Ok(alloc::vec::Vec::new())
	}

	fn removexattr(&self, inode: &Inode, name: &str) -> Result<()> {
		Err(Error::EPERM)
	}
}

/// Mount devfs
pub fn mount_devfs(_dev_name: &str, _flags: u32, _data: Option<&str>) -> Result<Arc<SuperBlock>> {
	let mut sb = SuperBlock::new("devfs")?;
	sb.s_magic = 0x1373; // DEVFS magic

	let devfs = Box::leak(Box::new(DevFs::new()));
	sb.s_fs_info = Some(devfs as *mut DevFs as *mut u8);

	// Create root inode
	let root_inode = Arc::new({
		let mut inode = Inode::new(1, mode::S_IFDIR | 0o755);
		inode.set_operations(Arc::new(DevFsInodeOps::new(devfs)));
		inode
	});

	let root_dentry = Arc::new(Dentry::new(String::from("/"), Some(root_inode)));
	sb.s_root = Some(root_dentry);

	Ok(Arc::new(sb))
}

/// Register devfs filesystem
pub fn register_devfs() -> Result<()> {
	let devfs_type = FileSystemType::new(
		String::from("devfs"),
		|_fstype, flags, _dev_name, data| mount_devfs(_dev_name, flags, data),
		|_sb| Ok(()),
	);

	// TODO: Register with VFS
	crate::console::print_info("Registered devfs filesystem\n");
	Ok(())
}
