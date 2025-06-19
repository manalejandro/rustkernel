// SPDX-License-Identifier: GPL-2.0

//! Inode abstraction - Linux compatible

use alloc::string::String;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::device::DeviceNumber;
use crate::error::{Error, Result};
use crate::sync::{Arc, Mutex};
use crate::time::{get_current_time, TimeSpec};

/// Inode structure - similar to Linux struct inode
#[derive(Debug)]
pub struct Inode {
	/// Inode number
	pub i_ino: u64,
	/// File mode and type
	pub i_mode: AtomicU32,
	/// Number of hard links
	pub i_nlink: AtomicU32,
	/// User ID
	pub i_uid: AtomicU32,
	/// Group ID
	pub i_gid: AtomicU32,
	/// Device number (for device files)
	pub i_rdev: DeviceNumber,
	/// File size
	pub i_size: AtomicU64,
	/// Block size
	pub i_blksize: u32,
	/// Number of blocks
	pub i_blocks: AtomicU64,
	/// Access time
	pub i_atime: Mutex<TimeSpec>,
	/// Modification time
	pub i_mtime: Mutex<TimeSpec>,
	/// Status change time
	pub i_ctime: Mutex<TimeSpec>,
	/// Inode operations
	pub i_op: Option<Arc<dyn InodeOperations>>,
	/// File operations (for regular files)
	pub i_fop: Option<Arc<dyn super::FileOperations>>,
	/// Superblock this inode belongs to
	pub i_sb: Option<Arc<super::SuperBlock>>,
	/// Private data
	pub private_data: Option<*mut u8>,
	/// Reference count
	pub refcount: AtomicU32,
	/// Inode flags
	pub i_flags: AtomicU32,
}

impl Inode {
	/// Create a new inode
	pub fn new(ino: u64, mode: u32) -> Self {
		let now = get_current_time();
		Self {
			i_ino: ino,
			i_mode: AtomicU32::new(mode),
			i_nlink: AtomicU32::new(1),
			i_uid: AtomicU32::new(0),
			i_gid: AtomicU32::new(0),
			i_rdev: DeviceNumber::new(0, 0),
			i_size: AtomicU64::new(0),
			i_blksize: 4096,
			i_blocks: AtomicU64::new(0),
			i_atime: Mutex::new(now),
			i_mtime: Mutex::new(now),
			i_ctime: Mutex::new(now),
			i_op: None,
			i_fop: None,
			i_sb: None,
			private_data: None,
			refcount: AtomicU32::new(1),
			i_flags: AtomicU32::new(0),
		}
	}

	/// Set inode operations
	pub fn set_operations(&mut self, ops: Arc<dyn InodeOperations>) {
		self.i_op = Some(ops);
	}

	/// Set file operations
	pub fn set_file_operations(&mut self, ops: Arc<dyn super::FileOperations>) {
		self.i_fop = Some(ops);
	}

	/// Get file statistics
	pub fn stat(&self) -> Result<super::KStat> {
		let atime = self.i_atime.lock();
		let mtime = self.i_mtime.lock();
		let ctime = self.i_ctime.lock();

		Ok(super::KStat {
			st_dev: if let Some(ref sb) = self.i_sb {
				sb.s_dev.as_raw()
			} else {
				0
			},
			st_ino: self.i_ino,
			st_nlink: self.i_nlink.load(Ordering::Relaxed) as u64,
			st_mode: self.i_mode.load(Ordering::Relaxed),
			st_uid: self.i_uid.load(Ordering::Relaxed),
			st_gid: self.i_gid.load(Ordering::Relaxed),
			st_rdev: self.i_rdev.as_raw(),
			st_size: self.i_size.load(Ordering::Relaxed) as i64,
			st_blksize: self.i_blksize as u64,
			st_blocks: self.i_blocks.load(Ordering::Relaxed),
			st_atime: atime.tv_sec,
			st_atime_nsec: atime.tv_nsec,
			st_mtime: mtime.tv_sec,
			st_mtime_nsec: mtime.tv_nsec,
			st_ctime: ctime.tv_sec,
			st_ctime_nsec: ctime.tv_nsec,
		})
	}

	/// Check if inode is a regular file
	pub fn is_regular(&self) -> bool {
		let mode = self.i_mode.load(Ordering::Relaxed);
		super::mode::s_isreg(mode)
	}

	/// Check if inode is a directory
	pub fn is_directory(&self) -> bool {
		let mode = self.i_mode.load(Ordering::Relaxed);
		super::mode::s_isdir(mode)
	}

	/// Check if inode is a character device
	pub fn is_char_device(&self) -> bool {
		let mode = self.i_mode.load(Ordering::Relaxed);
		super::mode::s_ischr(mode)
	}

	/// Check if inode is a block device
	pub fn is_block_device(&self) -> bool {
		let mode = self.i_mode.load(Ordering::Relaxed);
		super::mode::s_isblk(mode)
	}

	/// Update access time
	pub fn update_atime(&self) {
		let mut atime = self.i_atime.lock();
		*atime = get_current_time();
	}

	/// Update modification time
	pub fn update_mtime(&self) {
		let mut mtime = self.i_mtime.lock();
		*mtime = get_current_time();
	}

	/// Update status change time
	pub fn update_ctime(&self) {
		let mut ctime = self.i_ctime.lock();
		*ctime = get_current_time();
	}

	/// Set file size
	pub fn set_size(&self, size: u64) {
		self.i_size.store(size, Ordering::Relaxed);
		self.update_mtime();
		self.update_ctime();
	}

	/// Get file size
	pub fn get_size(&self) -> u64 {
		self.i_size.load(Ordering::Relaxed)
	}

	/// Increment reference count
	pub fn iget(&self) {
		self.refcount.fetch_add(1, Ordering::Relaxed);
	}

	/// Decrement reference count
	pub fn iput(&self) {
		let old_count = self.refcount.fetch_sub(1, Ordering::Relaxed);
		if old_count == 1 {
			// Last reference, inode should be cleaned up
			// TODO: Call destroy_inode operation if present
		}
	}

	/// Create a new file in this directory
	pub fn create(&self, name: &str, mode: u32) -> Result<Arc<Inode>> {
		if let Some(ref ops) = self.i_op {
			ops.create(self, name, mode)
		} else {
			Err(Error::ENOSYS)
		}
	}

	/// Look up a file in this directory
	pub fn lookup(&self, name: &str) -> Result<Arc<Inode>> {
		if let Some(ref ops) = self.i_op {
			ops.lookup(self, name)
		} else {
			Err(Error::ENOSYS)
		}
	}

	/// Create a directory
	pub fn mkdir(&self, name: &str, mode: u32) -> Result<Arc<Inode>> {
		if let Some(ref ops) = self.i_op {
			ops.mkdir(self, name, mode)
		} else {
			Err(Error::ENOSYS)
		}
	}

	/// Remove a file
	pub fn unlink(&self, name: &str) -> Result<()> {
		if let Some(ref ops) = self.i_op {
			ops.unlink(self, name)
		} else {
			Err(Error::ENOSYS)
		}
	}

	/// Remove a directory
	pub fn rmdir(&self, name: &str) -> Result<()> {
		if let Some(ref ops) = self.i_op {
			ops.rmdir(self, name)
		} else {
			Err(Error::ENOSYS)
		}
	}
}

unsafe impl Send for Inode {}
unsafe impl Sync for Inode {}

/// Inode operations trait - similar to Linux inode_operations
pub trait InodeOperations: Send + Sync + core::fmt::Debug {
	/// Look up a file in directory
	fn lookup(&self, dir: &Inode, name: &str) -> Result<Arc<Inode>>;

	/// Create a new file
	fn create(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>>;

	/// Create a directory
	fn mkdir(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>>;

	/// Remove a file
	fn unlink(&self, dir: &Inode, name: &str) -> Result<()>;

	/// Remove a directory
	fn rmdir(&self, dir: &Inode, name: &str) -> Result<()>;

	/// Create a symbolic link
	fn symlink(&self, dir: &Inode, name: &str, target: &str) -> Result<Arc<Inode>>;

	/// Rename a file
	fn rename(
		&self,
		old_dir: &Inode,
		old_name: &str,
		new_dir: &Inode,
		new_name: &str,
	) -> Result<()>;

	/// Set attributes
	fn setattr(&self, inode: &Inode, attr: &InodeAttr) -> Result<()>;

	/// Get attributes
	fn getattr(&self, inode: &Inode) -> Result<InodeAttr>;

	/// Read symbolic link
	fn readlink(&self, inode: &Inode) -> Result<String>;

	/// Follow symbolic link
	fn follow_link(&self, inode: &Inode) -> Result<Arc<Inode>>;

	/// Truncate file
	fn truncate(&self, inode: &Inode, size: u64) -> Result<()>;

	/// Get extended attribute
	fn getxattr(&self, inode: &Inode, name: &str) -> Result<alloc::vec::Vec<u8>>;

	/// Set extended attribute
	fn setxattr(&self, inode: &Inode, name: &str, value: &[u8], flags: u32) -> Result<()>;

	/// List extended attributes
	fn listxattr(&self, inode: &Inode) -> Result<alloc::vec::Vec<String>>;

	/// Remove extended attribute
	fn removexattr(&self, inode: &Inode, name: &str) -> Result<()>;
}

/// Inode attributes structure
#[derive(Debug, Clone, Copy)]
pub struct InodeAttr {
	pub mode: Option<u32>,
	pub uid: Option<u32>,
	pub gid: Option<u32>,
	pub size: Option<u64>,
	pub atime: Option<TimeSpec>,
	pub mtime: Option<TimeSpec>,
	pub ctime: Option<TimeSpec>,
}

impl InodeAttr {
	pub fn new() -> Self {
		Self {
			mode: None,
			uid: None,
			gid: None,
			size: None,
			atime: None,
			mtime: None,
			ctime: None,
		}
	}

	pub fn with_mode(mut self, mode: u32) -> Self {
		self.mode = Some(mode);
		self
	}

	pub fn with_size(mut self, size: u64) -> Self {
		self.size = Some(size);
		self
	}
}

/// Generic inode operations for simple filesystems
#[derive(Debug)]
pub struct GenericInodeOps;

impl InodeOperations for GenericInodeOps {
	fn lookup(&self, dir: &Inode, name: &str) -> Result<Arc<Inode>> {
		Err(Error::ENOENT)
	}

	fn create(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>> {
		Err(Error::ENOSYS)
	}

	fn mkdir(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>> {
		Err(Error::ENOSYS)
	}

	fn unlink(&self, dir: &Inode, name: &str) -> Result<()> {
		Err(Error::ENOSYS)
	}

	fn rmdir(&self, dir: &Inode, name: &str) -> Result<()> {
		Err(Error::ENOSYS)
	}

	fn symlink(&self, dir: &Inode, name: &str, target: &str) -> Result<Arc<Inode>> {
		Err(Error::ENOSYS)
	}

	fn rename(
		&self,
		old_dir: &Inode,
		old_name: &str,
		new_dir: &Inode,
		new_name: &str,
	) -> Result<()> {
		Err(Error::ENOSYS)
	}

	fn setattr(&self, inode: &Inode, attr: &InodeAttr) -> Result<()> {
		// Apply basic attributes
		if let Some(mode) = attr.mode {
			inode.i_mode.store(mode, Ordering::Relaxed);
		}
		if let Some(uid) = attr.uid {
			inode.i_uid.store(uid, Ordering::Relaxed);
		}
		if let Some(gid) = attr.gid {
			inode.i_gid.store(gid, Ordering::Relaxed);
		}
		if let Some(size) = attr.size {
			inode.set_size(size);
		}
		if let Some(atime) = attr.atime {
			*inode.i_atime.lock() = atime;
		}
		if let Some(mtime) = attr.mtime {
			*inode.i_mtime.lock() = mtime;
		}
		if let Some(ctime) = attr.ctime {
			*inode.i_ctime.lock() = ctime;
		}
		Ok(())
	}

	fn getattr(&self, inode: &Inode) -> Result<InodeAttr> {
		Ok(InodeAttr {
			mode: Some(inode.i_mode.load(Ordering::Relaxed)),
			uid: Some(inode.i_uid.load(Ordering::Relaxed)),
			gid: Some(inode.i_gid.load(Ordering::Relaxed)),
			size: Some(inode.i_size.load(Ordering::Relaxed)),
			atime: Some(*inode.i_atime.lock()),
			mtime: Some(*inode.i_mtime.lock()),
			ctime: Some(*inode.i_ctime.lock()),
		})
	}

	fn readlink(&self, inode: &Inode) -> Result<String> {
		Err(Error::EINVAL)
	}

	fn follow_link(&self, inode: &Inode) -> Result<Arc<Inode>> {
		Err(Error::EINVAL)
	}

	fn truncate(&self, inode: &Inode, size: u64) -> Result<()> {
		inode.set_size(size);
		Ok(())
	}

	fn getxattr(&self, inode: &Inode, name: &str) -> Result<alloc::vec::Vec<u8>> {
		Err(Error::ENODATA)
	}

	fn setxattr(&self, inode: &Inode, name: &str, value: &[u8], flags: u32) -> Result<()> {
		Err(Error::ENOSYS)
	}

	fn listxattr(&self, inode: &Inode) -> Result<alloc::vec::Vec<String>> {
		Ok(alloc::vec::Vec::new())
	}

	fn removexattr(&self, inode: &Inode, name: &str) -> Result<()> {
		Err(Error::ENODATA)
	}
}
