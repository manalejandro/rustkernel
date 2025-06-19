// SPDX-License-Identifier: GPL-2.0

//! Superblock abstraction - Linux compatible

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::device::DeviceNumber;
use crate::error::Result;
use crate::sync::{Arc, Mutex};

/// Superblock structure - similar to Linux struct super_block
#[derive(Debug)]
pub struct SuperBlock {
	/// Device number
	pub s_dev: DeviceNumber,
	/// Block size
	pub s_blocksize: u32,
	/// Block size bits
	pub s_blocksize_bits: u8,
	/// Maximum file size
	pub s_maxbytes: u64,
	/// File system type
	pub s_type: Option<Arc<FileSystemType>>,
	/// Superblock operations
	pub s_op: Option<Arc<dyn SuperOperations>>,
	/// Root dentry
	pub s_root: Option<Arc<super::Dentry>>,
	/// Mount point
	pub s_mount: Option<Arc<super::VfsMount>>,
	/// File system flags
	pub s_flags: AtomicU32,
	/// File system magic number
	pub s_magic: u32,
	/// List of inodes
	pub s_inodes: Mutex<Vec<Arc<super::Inode>>>,
	/// Next inode number
	pub s_next_ino: AtomicU64,
	/// Private data
	pub s_fs_info: Option<*mut u8>,
	/// Dirty inodes
	pub s_dirty: Mutex<Vec<Arc<super::Inode>>>,
	/// Reference count
	pub s_count: AtomicU32,
	/// File system name
	pub s_id: String,
}

impl SuperBlock {
	/// Create a new superblock
	pub fn new(fstype: &str) -> Result<Self> {
		Ok(Self {
			s_dev: DeviceNumber::new(0, 0),
			s_blocksize: 4096,
			s_blocksize_bits: 12,
			s_maxbytes: 0x7fffffffffffffff,
			s_type: None,
			s_op: None,
			s_root: None,
			s_mount: None,
			s_flags: AtomicU32::new(0),
			s_magic: 0,
			s_inodes: Mutex::new(Vec::new()),
			s_next_ino: AtomicU64::new(1),
			s_fs_info: None,
			s_dirty: Mutex::new(Vec::new()),
			s_count: AtomicU32::new(1),
			s_id: String::from(fstype),
		})
	}

	/// Set superblock operations
	pub fn set_operations(&mut self, ops: Arc<dyn SuperOperations>) {
		self.s_op = Some(ops);
	}

	/// Allocate a new inode
	pub fn alloc_inode(&self, mode: u32) -> Result<Arc<super::Inode>> {
		let ino = self.s_next_ino.fetch_add(1, Ordering::Relaxed);
		let mut inode = super::Inode::new(ino, mode);
		inode.i_sb = Some(Arc::new(unsafe {
			// SAFETY: We're creating a weak reference to avoid cycles
			core::ptr::read(self as *const Self)
		}));

		let inode = Arc::new(inode);

		// Add to inode list
		let mut inodes = self.s_inodes.lock();
		inodes.push(inode.clone());

		Ok(inode)
	}

	/// Write superblock to disk
	pub fn write_super(&self) -> Result<()> {
		if let Some(ref ops) = self.s_op {
			ops.write_super(self)
		} else {
			Ok(())
		}
	}

	/// Put superblock (decrement reference count)
	pub fn put_super(&self) -> Result<()> {
		let old_count = self.s_count.fetch_sub(1, Ordering::Relaxed);
		if old_count == 1 {
			// Last reference, cleanup
			if let Some(ref ops) = self.s_op {
				ops.put_super(self)?;
			}
		}
		Ok(())
	}

	/// Get superblock statistics
	pub fn statfs(&self) -> Result<super::KStatFs> {
		if let Some(ref ops) = self.s_op {
			ops.statfs(self)
		} else {
			// Default statistics
			Ok(super::KStatFs {
				f_type: self.s_magic as u64,
				f_bsize: self.s_blocksize as u64,
				f_blocks: 0,
				f_bfree: 0,
				f_bavail: 0,
				f_files: 0,
				f_ffree: 0,
				f_fsid: [0, 0],
				f_namelen: super::NAME_MAX as u64,
				f_frsize: self.s_blocksize as u64,
				f_flags: self.s_flags.load(Ordering::Relaxed) as u64,
				f_spare: [0; 4],
			})
		}
	}

	/// Sync filesystem
	pub fn sync_fs(&self, wait: bool) -> Result<()> {
		if let Some(ref ops) = self.s_op {
			ops.sync_fs(self, wait)
		} else {
			Ok(())
		}
	}

	/// Freeze filesystem
	pub fn freeze_fs(&self) -> Result<()> {
		if let Some(ref ops) = self.s_op {
			ops.freeze_fs(self)
		} else {
			Ok(())
		}
	}

	/// Unfreeze filesystem
	pub fn unfreeze_fs(&self) -> Result<()> {
		if let Some(ref ops) = self.s_op {
			ops.unfreeze_fs(self)
		} else {
			Ok(())
		}
	}

	/// Mark inode as dirty
	pub fn mark_dirty(&self, inode: Arc<super::Inode>) {
		let mut dirty = self.s_dirty.lock();
		dirty.push(inode);
	}

	/// Write back dirty inodes
	pub fn write_dirty(&self) -> Result<()> {
		let mut dirty = self.s_dirty.lock();
		for inode in dirty.drain(..) {
			// TODO: Write inode to disk
		}
		Ok(())
	}
}

unsafe impl Send for SuperBlock {}
unsafe impl Sync for SuperBlock {}

/// Superblock operations trait - similar to Linux super_operations
pub trait SuperOperations: Send + Sync + core::fmt::Debug {
	/// Allocate inode
	fn alloc_inode(&self, sb: &SuperBlock) -> Result<Arc<super::Inode>>;

	/// Destroy inode
	fn destroy_inode(&self, inode: &super::Inode) -> Result<()>;

	/// Write inode
	fn write_inode(&self, inode: &super::Inode, sync: bool) -> Result<()>;

	/// Delete inode
	fn delete_inode(&self, inode: &super::Inode) -> Result<()>;

	/// Put superblock
	fn put_super(&self, sb: &SuperBlock) -> Result<()>;

	/// Write superblock
	fn write_super(&self, sb: &SuperBlock) -> Result<()>;

	/// Sync filesystem
	fn sync_fs(&self, sb: &SuperBlock, wait: bool) -> Result<()>;

	/// Freeze filesystem
	fn freeze_fs(&self, sb: &SuperBlock) -> Result<()>;

	/// Unfreeze filesystem
	fn unfreeze_fs(&self, sb: &SuperBlock) -> Result<()>;

	/// Get filesystem statistics
	fn statfs(&self, sb: &SuperBlock) -> Result<super::KStatFs>;

	/// Remount filesystem
	fn remount_fs(&self, sb: &SuperBlock, flags: u32, data: Option<&str>) -> Result<()>;

	/// Show mount options
	fn show_options(&self, sb: &SuperBlock) -> Result<String>;
}

/// File system type structure - similar to Linux file_system_type
#[derive(Debug)]
pub struct FileSystemType {
	/// File system name
	pub name: String,
	/// File system flags
	pub fs_flags: u32,
	/// Mount function
	pub mount: fn(
		fstype: &FileSystemType,
		flags: u32,
		dev_name: &str,
		data: Option<&str>,
	) -> Result<Arc<SuperBlock>>,
	/// Kill superblock function
	pub kill_sb: fn(sb: &SuperBlock) -> Result<()>,
	/// Owner module
	pub owner: Option<&'static str>,
}

impl FileSystemType {
	/// Create a new file system type
	pub fn new(
		name: String,
		mount: fn(&FileSystemType, u32, &str, Option<&str>) -> Result<Arc<SuperBlock>>,
		kill_sb: fn(&SuperBlock) -> Result<()>,
	) -> Self {
		Self {
			name,
			fs_flags: 0,
			mount,
			kill_sb,
			owner: None,
		}
	}

	/// Mount this filesystem type
	pub fn do_mount(
		&self,
		flags: u32,
		dev_name: &str,
		data: Option<&str>,
	) -> Result<Arc<SuperBlock>> {
		(self.mount)(self, flags, dev_name, data)
	}

	/// Kill superblock
	pub fn do_kill_sb(&self, sb: &SuperBlock) -> Result<()> {
		(self.kill_sb)(sb)
	}
}

/// Generic superblock operations
#[derive(Debug)]
pub struct GenericSuperOps;

impl SuperOperations for GenericSuperOps {
	fn alloc_inode(&self, sb: &SuperBlock) -> Result<Arc<super::Inode>> {
		sb.alloc_inode(0o644)
	}

	fn destroy_inode(&self, inode: &super::Inode) -> Result<()> {
		Ok(())
	}

	fn write_inode(&self, inode: &super::Inode, sync: bool) -> Result<()> {
		Ok(())
	}

	fn delete_inode(&self, inode: &super::Inode) -> Result<()> {
		Ok(())
	}

	fn put_super(&self, sb: &SuperBlock) -> Result<()> {
		Ok(())
	}

	fn write_super(&self, sb: &SuperBlock) -> Result<()> {
		Ok(())
	}

	fn sync_fs(&self, sb: &SuperBlock, wait: bool) -> Result<()> {
		sb.write_dirty()
	}

	fn freeze_fs(&self, sb: &SuperBlock) -> Result<()> {
		Ok(())
	}

	fn unfreeze_fs(&self, sb: &SuperBlock) -> Result<()> {
		Ok(())
	}

	fn statfs(&self, sb: &SuperBlock) -> Result<super::KStatFs> {
		sb.statfs()
	}

	fn remount_fs(&self, sb: &SuperBlock, flags: u32, data: Option<&str>) -> Result<()> {
		sb.s_flags.store(flags, Ordering::Relaxed);
		Ok(())
	}

	fn show_options(&self, sb: &SuperBlock) -> Result<String> {
		Ok(String::new())
	}
}

/// File system flags
pub const FS_REQUIRES_DEV: u32 = 1;
pub const FS_BINARY_MOUNTDATA: u32 = 2;
pub const FS_HAS_SUBTYPE: u32 = 4;
pub const FS_USERNS_MOUNT: u32 = 8;
pub const FS_DISALLOW_NOTIFY_PERM: u32 = 16;
pub const FS_RENAME_DOES_D_MOVE: u32 = 32;

/// Mount flags
pub const MS_RDONLY: u32 = 1;
pub const MS_NOSUID: u32 = 2;
pub const MS_NODEV: u32 = 4;
pub const MS_NOEXEC: u32 = 8;
pub const MS_SYNCHRONOUS: u32 = 16;
pub const MS_REMOUNT: u32 = 32;
pub const MS_MANDLOCK: u32 = 64;
pub const MS_DIRSYNC: u32 = 128;
pub const MS_NOATIME: u32 = 1024;
pub const MS_NODIRATIME: u32 = 2048;
pub const MS_BIND: u32 = 4096;
pub const MS_MOVE: u32 = 8192;
pub const MS_REC: u32 = 16384;
pub const MS_SILENT: u32 = 32768;
pub const MS_POSIXACL: u32 = 1 << 16;
pub const MS_UNBINDABLE: u32 = 1 << 17;
pub const MS_PRIVATE: u32 = 1 << 18;
pub const MS_SLAVE: u32 = 1 << 19;
pub const MS_SHARED: u32 = 1 << 20;
pub const MS_RELATIME: u32 = 1 << 21;
pub const MS_KERNMOUNT: u32 = 1 << 22;
pub const MS_I_VERSION: u32 = 1 << 23;
pub const MS_STRICTATIME: u32 = 1 << 24;
