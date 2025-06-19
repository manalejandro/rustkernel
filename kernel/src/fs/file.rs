// SPDX-License-Identifier: GPL-2.0

//! File abstraction - Linux compatible

use alloc::string::String;
use core::sync::atomic::{AtomicI64, AtomicU32, Ordering};

use crate::error::{Error, Result};
// use crate::types::*;  // Commented out - unused for now
use crate::memory::UserSlicePtr; // Remove UserPtr since it's unused
use crate::sync::Arc; // Remove Mutex and RwLock since they're unused

/// File structure - similar to Linux struct file
#[derive(Debug)]
pub struct File {
	/// File operations
	pub f_op: Option<Arc<dyn FileOperations>>,
	/// Current file position
	pub pos: AtomicI64,
	/// File flags
	pub flags: AtomicU32,
	/// File mode
	pub mode: u32,
	/// Associated inode
	pub inode: Option<Arc<super::Inode>>,
	/// Associated dentry
	pub dentry: Option<Arc<super::Dentry>>,
	/// Private data for file operations
	pub private_data: Option<*mut u8>,
	/// File path (for debugging/proc)
	pub path: String,
	/// Reference count
	pub refcount: AtomicU32,
}

impl File {
	/// Create a new file
	pub fn new(path: &str, flags: u32, mode: u32) -> Result<Self> {
		Ok(Self {
			f_op: None,
			pos: AtomicI64::new(0),
			flags: AtomicU32::new(flags),
			mode,
			inode: None,
			dentry: None,
			private_data: None,
			path: String::from(path),
			refcount: AtomicU32::new(1),
		})
	}

	/// Set file operations
	pub fn set_operations(&mut self, ops: Arc<dyn FileOperations>) {
		self.f_op = Some(ops);
	}

	/// Read from file
	pub fn read(&self, buf: UserSlicePtr, count: usize) -> Result<isize> {
		if let Some(ref ops) = self.f_op {
			ops.read(self, buf, count)
		} else {
			Err(Error::ENOSYS)
		}
	}

	/// Write to file
	pub fn write(&self, buf: UserSlicePtr, count: usize) -> Result<isize> {
		if let Some(ref ops) = self.f_op {
			ops.write(self, buf, count)
		} else {
			Err(Error::ENOSYS)
		}
	}

	/// Seek within file
	pub fn seek(&self, offset: i64, whence: i32) -> Result<i64> {
		if let Some(ref ops) = self.f_op {
			let new_pos = ops.seek(self, offset, whence)?;
			self.pos.store(new_pos, Ordering::Relaxed);
			Ok(new_pos)
		} else {
			Err(Error::ENOSYS)
		}
	}

	/// Get file status
	pub fn stat(&self) -> Result<super::KStat> {
		if let Some(ref inode) = self.inode {
			inode.stat()
		} else {
			// Create a basic stat structure
			Ok(super::KStat {
				st_dev: 0,
				st_ino: 0,
				st_nlink: 1,
				st_mode: self.mode,
				st_uid: 0,
				st_gid: 0,
				st_rdev: 0,
				st_size: 0,
				st_blksize: 4096,
				st_blocks: 0,
				st_atime: 0,
				st_atime_nsec: 0,
				st_mtime: 0,
				st_mtime_nsec: 0,
				st_ctime: 0,
				st_ctime_nsec: 0,
			})
		}
	}

	/// Perform ioctl operation
	pub fn ioctl(&self, cmd: u32, arg: usize) -> Result<isize> {
		if let Some(ref ops) = self.f_op {
			ops.ioctl(self, cmd, arg)
		} else {
			Err(Error::ENOTTY)
		}
	}

	/// Memory map file
	pub fn mmap(&self, vma: &mut crate::memory::VmaArea) -> Result<()> {
		if let Some(ref ops) = self.f_op {
			ops.mmap(self, vma)
		} else {
			Err(Error::ENODEV)
		}
	}

	/// Sync file to storage
	pub fn fsync(&self, datasync: bool) -> Result<()> {
		if let Some(ref ops) = self.f_op {
			ops.fsync(self, datasync)
		} else {
			Ok(()) // No-op for files without sync
		}
	}

	/// Poll file for events
	pub fn poll(&self, wait: &mut super::PollWait) -> Result<u32> {
		if let Some(ref ops) = self.f_op {
			ops.poll(self, wait)
		} else {
			Ok(super::POLLIN | super::POLLOUT) // Always ready
		}
	}

	/// Get current file position
	pub fn get_pos(&self) -> i64 {
		self.pos.load(Ordering::Relaxed)
	}

	/// Set file position
	pub fn set_pos(&self, pos: i64) {
		self.pos.store(pos, Ordering::Relaxed);
	}

	/// Get file flags
	pub fn get_flags(&self) -> u32 {
		self.flags.load(Ordering::Relaxed)
	}

	/// Check if file is readable
	pub fn is_readable(&self) -> bool {
		let flags = self.get_flags();
		(flags & super::flags::O_ACCMODE) != super::flags::O_WRONLY
	}

	/// Check if file is writable
	pub fn is_writable(&self) -> bool {
		let flags = self.get_flags();
		(flags & super::flags::O_ACCMODE) != super::flags::O_RDONLY
	}

	/// Increment reference count
	pub fn get_file(&self) -> Result<()> {
		self.refcount.fetch_add(1, Ordering::Relaxed);
		Ok(())
	}

	/// Decrement reference count (fput equivalent)
	pub fn put_file(&self) -> Result<()> {
		let old_count = self.refcount.fetch_sub(1, Ordering::Relaxed);
		if old_count == 1 {
			// Last reference, file should be cleaned up
			// TODO: Call release operation if present
		}
		Ok(())
	}
}

unsafe impl Send for File {}
unsafe impl Sync for File {}

/// File operations trait - similar to Linux file_operations
pub trait FileOperations: Send + Sync + core::fmt::Debug {
	/// Read from file
	fn read(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize>;

	/// Write to file
	fn write(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize>;

	/// Seek within file
	fn seek(&self, file: &File, offset: i64, whence: i32) -> Result<i64>;

	/// I/O control
	fn ioctl(&self, file: &File, cmd: u32, arg: usize) -> Result<isize>;

	/// Memory map
	fn mmap(&self, file: &File, vma: &mut crate::memory::VmaArea) -> Result<()>;

	/// Sync file
	fn fsync(&self, file: &File, datasync: bool) -> Result<()>;

	/// Poll for events
	fn poll(&self, file: &File, wait: &mut super::PollWait) -> Result<u32>;

	/// Open file (optional)
	fn open(&self, inode: &super::Inode, file: &File) -> Result<()> {
		Ok(())
	}

	/// Release file (optional)
	fn release(&self, inode: &super::Inode, file: &File) -> Result<()> {
		Ok(())
	}

	/// Flush file (optional)
	fn flush(&self, file: &File) -> Result<()> {
		Ok(())
	}

	/// Lock file (optional)
	fn lock(&self, file: &File, cmd: i32) -> Result<()> {
		Err(Error::ENOSYS)
	}

	/// Read directory entries (optional)
	fn readdir(&self, file: &File, ctx: &mut super::DirContext) -> Result<()> {
		Err(Error::ENOTDIR)
	}
}

/// Directory context for readdir operations
pub struct DirContext {
	/// Current position in directory
	pub pos: i64,
	/// Entries collected so far
	pub entries: alloc::vec::Vec<super::DirEntry>,
}

impl DirContext {
	pub fn new(pos: i64) -> Self {
		Self {
			pos,
			entries: alloc::vec::Vec::new(),
		}
	}

	/// Add a directory entry
	pub fn add_entry(&mut self, ino: u64, name: &str, d_type: u8) {
		let entry = super::DirEntry {
			ino,
			off: self.pos,
			reclen: (core::mem::size_of::<super::DirEntry>() + name.len() + 1) as u16,
			name: String::from(name),
			d_type,
		};
		self.entries.push(entry);
		self.pos += 1;
	}
}
