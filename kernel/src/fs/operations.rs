// SPDX-License-Identifier: GPL-2.0

//! Various VFS operations and utilities

use crate::error::{Error, Result};
use crate::memory::UserSlicePtr;
use crate::sync::Arc;

/// Address space operations trait - similar to Linux address_space_operations
pub trait AddressSpaceOperations: Send + Sync {
	/// Write a page
	fn writepage(&self, page: &crate::memory::Page) -> Result<()>;

	/// Read a page
	fn readpage(&self, file: Option<&super::File>, page: &crate::memory::Page) -> Result<()>;

	/// Sync pages
	fn sync_page(&self, page: &crate::memory::Page) -> Result<()>;

	/// Write pages
	fn writepages(&self, mapping: &AddressSpace, wbc: &WritebackControl) -> Result<()>;

	/// Set page dirty
	fn set_page_dirty(&self, page: &crate::memory::Page) -> Result<bool>;

	/// Read pages ahead
	fn readpages(
		&self,
		file: Option<&super::File>,
		pages: &[&crate::memory::Page],
	) -> Result<()>;

	/// Write begin
	fn write_begin(&self, file: &super::File, pos: u64, len: u32) -> Result<()>;

	/// Write end
	fn write_end(&self, file: &super::File, pos: u64, len: u32, copied: u32) -> Result<u32>;

	/// Direct I/O
	fn direct_io(
		&self,
		file: &super::File,
		pos: u64,
		buf: UserSlicePtr,
		len: usize,
		write: bool,
	) -> Result<isize>;
}

/// Address space structure
pub struct AddressSpace {
	/// Host inode
	pub host: Option<Arc<super::Inode>>,
	/// Address space operations
	pub a_ops: Option<Arc<dyn AddressSpaceOperations>>,
	/// Number of pages
	pub nrpages: core::sync::atomic::AtomicUsize,
	/// Flags
	pub flags: core::sync::atomic::AtomicU32,
	/// Private data
	pub private_data: Option<*mut u8>,
}

impl AddressSpace {
	pub fn new() -> Self {
		Self {
			host: None,
			a_ops: None,
			nrpages: core::sync::atomic::AtomicUsize::new(0),
			flags: core::sync::atomic::AtomicU32::new(0),
			private_data: None,
		}
	}
}

/// Writeback control structure
pub struct WritebackControl {
	pub start: u64,
	pub end: u64,
	pub sync_mode: WritebackSyncMode,
	pub nr_to_write: u64,
	pub tagged_writepages: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum WritebackSyncMode {
	None,
	All,
	Memory,
}

/// Generic file operations implementation
#[derive(Debug)]
pub struct GenericFileOperations;

impl super::FileOperations for GenericFileOperations {
	fn read(&self, file: &super::File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Generic read implementation using page cache
		// TODO: Implement proper page cache read
		Ok(0)
	}

	fn write(&self, file: &super::File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Generic write implementation using page cache
		// TODO: Implement proper page cache write
		Ok(count as isize)
	}

	fn seek(&self, file: &super::File, offset: i64, whence: i32) -> Result<i64> {
		let current_pos = file.get_pos();
		let new_pos = match whence {
			super::SEEK_SET => offset,
			super::SEEK_CUR => current_pos + offset,
			super::SEEK_END => {
				// Get file size from inode
				if let Some(ref inode) = file.inode {
					inode.get_size() as i64 + offset
				} else {
					offset
				}
			}
			_ => return Err(Error::EINVAL),
		};

		if new_pos < 0 {
			return Err(Error::EINVAL);
		}

		file.set_pos(new_pos);
		Ok(new_pos)
	}

	fn ioctl(&self, file: &super::File, cmd: u32, arg: usize) -> Result<isize> {
		Err(Error::ENOTTY)
	}

	fn mmap(&self, file: &super::File, vma: &mut crate::memory::VmaArea) -> Result<()> {
		// Generic mmap implementation
		// TODO: Implement proper memory mapping
		Err(Error::ENODEV)
	}

	fn fsync(&self, file: &super::File, datasync: bool) -> Result<()> {
		// Sync file data to storage
		if let Some(ref inode) = file.inode {
			// TODO: Sync inode and data blocks
		}
		Ok(())
	}

	fn poll(&self, file: &super::File, wait: &mut super::PollWait) -> Result<u32> {
		// Regular files are always ready for I/O
		Ok(super::POLLIN | super::POLLOUT)
	}

	fn readdir(&self, file: &super::File, ctx: &mut super::file::DirContext) -> Result<()> {
		// This shouldn't be called for regular files
		Err(Error::ENOTDIR)
	}
}

/// Directory file operations
#[derive(Debug)]
pub struct DirectoryOperations;

impl super::FileOperations for DirectoryOperations {
	fn read(&self, file: &super::File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Can't read directory as regular file
		Err(Error::EISDIR)
	}

	fn write(&self, file: &super::File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Can't write to directory as regular file
		Err(Error::EISDIR)
	}

	fn seek(&self, file: &super::File, offset: i64, whence: i32) -> Result<i64> {
		// Directory seeking
		match whence {
			super::SEEK_SET => {
				if offset < 0 {
					return Err(Error::EINVAL);
				}
				file.set_pos(offset);
				Ok(offset)
			}
			super::SEEK_CUR => {
				let new_pos = file.get_pos() + offset;
				if new_pos < 0 {
					return Err(Error::EINVAL);
				}
				file.set_pos(new_pos);
				Ok(new_pos)
			}
			super::SEEK_END => {
				// Seek to end of directory
				file.set_pos(i64::MAX);
				Ok(i64::MAX)
			}
			_ => Err(Error::EINVAL),
		}
	}

	fn ioctl(&self, file: &super::File, cmd: u32, arg: usize) -> Result<isize> {
		Err(Error::ENOTTY)
	}

	fn mmap(&self, file: &super::File, vma: &mut crate::memory::VmaArea) -> Result<()> {
		Err(Error::ENODEV)
	}

	fn fsync(&self, file: &super::File, datasync: bool) -> Result<()> {
		// Sync directory
		Ok(())
	}

	fn poll(&self, file: &super::File, wait: &mut super::PollWait) -> Result<u32> {
		// Directories are always ready
		Ok(super::POLLIN)
	}

	fn readdir(&self, file: &super::File, ctx: &mut super::file::DirContext) -> Result<()> {
		// Read directory entries
		if let Some(ref inode) = file.inode {
			if let Some(ref dentry) = file.dentry {
				let subdirs = dentry.d_subdirs.lock();
				for (i, child) in subdirs.iter().enumerate() {
					if i >= ctx.pos as usize {
						let d_type = if let Some(ref child_inode) =
							child.d_inode
						{
							let mode = child_inode.i_mode.load(core::sync::atomic::Ordering::Relaxed);
							if super::mode::s_isdir(mode) {
								super::DT_DIR
							} else if super::mode::s_isreg(mode) {
								super::DT_REG
							} else if super::mode::s_islnk(mode) {
								super::DT_LNK
							} else if super::mode::s_ischr(mode) {
								super::DT_CHR
							} else if super::mode::s_isblk(mode) {
								super::DT_BLK
							} else if super::mode::s_isfifo(mode) {
								super::DT_FIFO
							} else if super::mode::s_issock(mode) {
								super::DT_SOCK
							} else {
								super::DT_UNKNOWN
							}
						} else {
							super::DT_UNKNOWN
						};

						let ino = if let Some(ref child_inode) =
							child.d_inode
						{
							child_inode.i_ino
						} else {
							0
						};

						ctx.add_entry(ino, &child.d_name, d_type);
					}
				}
			}
		}
		Ok(())
	}
}

/// Special file operations (for device files)
#[derive(Debug)]
pub struct SpecialFileOperations;

impl super::FileOperations for SpecialFileOperations {
	fn read(&self, file: &super::File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Delegate to device driver
		if let Some(ref inode) = file.inode {
			if inode.is_char_device() || inode.is_block_device() {
				// TODO: Call device driver read function
				return Ok(0);
			}
		}
		Err(Error::ENODEV)
	}

	fn write(&self, file: &super::File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Delegate to device driver
		if let Some(ref inode) = file.inode {
			if inode.is_char_device() || inode.is_block_device() {
				// TODO: Call device driver write function
				return Ok(count as isize);
			}
		}
		Err(Error::ENODEV)
	}

	fn seek(&self, file: &super::File, offset: i64, whence: i32) -> Result<i64> {
		// Most device files don't support seeking
		Err(Error::ESPIPE)
	}

	fn ioctl(&self, file: &super::File, cmd: u32, arg: usize) -> Result<isize> {
		// Delegate to device driver
		if let Some(ref inode) = file.inode {
			if inode.is_char_device() || inode.is_block_device() {
				// TODO: Call device driver ioctl function
				return Ok(0);
			}
		}
		Err(Error::ENOTTY)
	}

	fn mmap(&self, file: &super::File, vma: &mut crate::memory::VmaArea) -> Result<()> {
		// Some device files support mmap
		Err(Error::ENODEV)
	}

	fn fsync(&self, file: &super::File, datasync: bool) -> Result<()> {
		// Nothing to sync for device files
		Ok(())
	}

	fn poll(&self, file: &super::File, wait: &mut super::PollWait) -> Result<u32> {
		// Delegate to device driver
		if let Some(ref inode) = file.inode {
			if inode.is_char_device() {
				// TODO: Call device driver poll function
				return Ok(super::POLLIN | super::POLLOUT);
			}
		}
		Ok(0)
	}
}

/// No-op address space operations
pub struct NoOpAddressSpaceOps;

impl AddressSpaceOperations for NoOpAddressSpaceOps {
	fn writepage(&self, page: &crate::memory::Page) -> Result<()> {
		Ok(())
	}

	fn readpage(&self, file: Option<&super::File>, page: &crate::memory::Page) -> Result<()> {
		Ok(())
	}

	fn sync_page(&self, page: &crate::memory::Page) -> Result<()> {
		Ok(())
	}

	fn writepages(&self, mapping: &AddressSpace, wbc: &WritebackControl) -> Result<()> {
		Ok(())
	}

	fn set_page_dirty(&self, page: &crate::memory::Page) -> Result<bool> {
		Ok(true)
	}

	fn readpages(
		&self,
		file: Option<&super::File>,
		pages: &[&crate::memory::Page],
	) -> Result<()> {
		Ok(())
	}

	fn write_begin(&self, file: &super::File, pos: u64, len: u32) -> Result<()> {
		Ok(())
	}

	fn write_end(&self, file: &super::File, pos: u64, len: u32, copied: u32) -> Result<u32> {
		Ok(copied)
	}

	fn direct_io(
		&self,
		file: &super::File,
		pos: u64,
		buf: UserSlicePtr,
		len: usize,
		write: bool,
	) -> Result<isize> {
		if write {
			Ok(len as isize)
		} else {
			Ok(0)
		}
	}
}

/// Helper functions for VFS operations

/// Get file operations for an inode
pub fn get_file_operations(inode: &super::Inode) -> Arc<dyn super::FileOperations> {
	if let Some(ref fop) = inode.i_fop {
		fop.clone()
	} else {
		let mode = inode.i_mode.load(core::sync::atomic::Ordering::Relaxed);
		if super::mode::s_isreg(mode) {
			Arc::new(GenericFileOperations)
		} else if super::mode::s_isdir(mode) {
			Arc::new(DirectoryOperations)
		} else if super::mode::s_ischr(mode) || super::mode::s_isblk(mode) {
			Arc::new(SpecialFileOperations)
		} else {
			Arc::new(super::GenericFileOps)
		}
	}
}

/// Check file permissions
pub fn check_permissions(inode: &super::Inode, mask: u32) -> Result<()> {
	// TODO: Implement proper permission checking
	// For now, allow all operations
	Ok(())
}

/// Update access time
pub fn update_atime(inode: &super::Inode) {
	inode.update_atime();
}

/// Update modification time
pub fn update_mtime(inode: &super::Inode) {
	inode.update_mtime();
}

/// Truncate file to specified size
pub fn do_truncate(inode: &super::Inode, size: u64) -> Result<()> {
	if let Some(ref ops) = inode.i_op {
		ops.truncate(inode, size)
	} else {
		inode.set_size(size);
		Ok(())
	}
}

/// Notify directory change
pub fn notify_change(inode: &super::Inode, attr: &super::inode::InodeAttr) -> Result<()> {
	if let Some(ref ops) = inode.i_op {
		ops.setattr(inode, attr)
	} else {
		Ok(())
	}
}
