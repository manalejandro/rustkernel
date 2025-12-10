// SPDX-License-Identifier: GPL-2.0

//! Virtual File System (VFS) - Linux compatible
//!
//! This module provides the core filesystem abstractions and compatibility
//! with Linux VFS operations.

pub mod dentry;
pub mod devfs;
pub mod file;
pub mod inode;
pub mod mode;
pub mod mount;
pub mod operations;
pub mod path;
pub mod procfs;
pub mod ramfs;
pub mod super_block; // Add mode module
		     // pub mod advanced;  // Advanced file system operations (removed for now)

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub use dentry::*;
pub use file::*;
pub use inode::*;
pub use mount::*;
pub use operations::*;
pub use path::*;
pub use super_block::*;

use crate::error::{Error, Result};
use crate::memory::{UserPtr, UserSlicePtr};
use crate::sync::{Arc, Mutex};

/// File access modes - Linux compatible
pub mod flags {
	pub const O_ACCMODE: u32 = 0o00000003;
	pub const O_RDONLY: u32 = 0o00000000;
	pub const O_WRONLY: u32 = 0o00000001;
	pub const O_RDWR: u32 = 0o00000002;
	pub const O_CREAT: u32 = 0o00000100;
	pub const O_EXCL: u32 = 0o00000200;
	pub const O_NOCTTY: u32 = 0o00000400;
	pub const O_TRUNC: u32 = 0o00001000;
	pub const O_APPEND: u32 = 0o00002000;
	pub const O_NONBLOCK: u32 = 0o00004000;
	pub const O_DSYNC: u32 = 0o00010000;
	pub const O_FASYNC: u32 = 0o00020000;
	pub const O_DIRECT: u32 = 0o00040000;
	pub const O_LARGEFILE: u32 = 0o00100000;
	pub const O_DIRECTORY: u32 = 0o00200000;
	pub const O_NOFOLLOW: u32 = 0o00400000;
	pub const O_NOATIME: u32 = 0o01000000;
	pub const O_CLOEXEC: u32 = 0o02000000;
	pub const O_SYNC: u32 = 0o04000000 | O_DSYNC;
	pub const O_PATH: u32 = 0o10000000;
	pub const O_TMPFILE: u32 = 0o20000000 | O_DIRECTORY;
}

/// Seek constants
pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;
pub const SEEK_DATA: i32 = 3;
pub const SEEK_HOLE: i32 = 4;

/// Maximum filename length
pub const NAME_MAX: usize = 255;

/// Maximum path length
pub const PATH_MAX: usize = 4096;

/// File system statistics structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KStatFs {
	pub f_type: u64,
	pub f_bsize: u64,
	pub f_blocks: u64,
	pub f_bfree: u64,
	pub f_bavail: u64,
	pub f_files: u64,
	pub f_ffree: u64,
	pub f_fsid: [u32; 2],
	pub f_namelen: u64,
	pub f_frsize: u64,
	pub f_flags: u64,
	pub f_spare: [u64; 4],
}

/// File attributes structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KStat {
	pub st_dev: u64,
	pub st_ino: u64,
	pub st_nlink: u64,
	pub st_mode: u32,
	pub st_uid: u32,
	pub st_gid: u32,
	pub st_rdev: u64,
	pub st_size: i64,
	pub st_blksize: u64,
	pub st_blocks: u64,
	pub st_atime: i64,
	pub st_atime_nsec: i64,
	pub st_mtime: i64,
	pub st_mtime_nsec: i64,
	pub st_ctime: i64,
	pub st_ctime_nsec: i64,
}

/// I/O vector for scatter-gather I/O
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IoVec {
	pub iov_base: *mut u8,
	pub iov_len: usize,
}

/// Directory entry type returned by readdir
#[derive(Debug, Clone)]
pub struct DirEntry {
	pub ino: u64,
	pub off: i64,
	pub reclen: u16,
	pub name: String,
	pub d_type: u8,
}

/// Directory entry types
pub const DT_UNKNOWN: u8 = 0;
pub const DT_FIFO: u8 = 1;
pub const DT_CHR: u8 = 2;
pub const DT_DIR: u8 = 4;
pub const DT_BLK: u8 = 6;
pub const DT_REG: u8 = 8;
pub const DT_LNK: u8 = 10;
pub const DT_SOCK: u8 = 12;
pub const DT_WHT: u8 = 14;

/// Global VFS state
static VFS: Mutex<Vfs> = Mutex::new(Vfs::new());

/// Global file descriptor table (simplified - in reality this would be
/// per-process)
static GLOBAL_FD_TABLE: Mutex<BTreeMap<i32, Arc<File>>> = Mutex::new(BTreeMap::new());
static NEXT_FD: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(3); // Start after stdin/stdout/stderr

/// Virtual File System state
pub struct Vfs {
	/// Mounted filesystems
	pub mounts: Vec<Arc<VfsMount>>,
	/// Root dentry
	pub root: Option<Arc<Dentry>>,
	/// File descriptor table (per-process will be separate)
	pub fd_table: BTreeMap<i32, Arc<File>>,
	/// Next file descriptor number
	pub next_fd: i32,
}

impl Vfs {
	const fn new() -> Self {
		Self {
			mounts: Vec::new(),
			root: None,
			fd_table: BTreeMap::new(),
			next_fd: 0,
		}
	}

	/// Mount a filesystem
	pub fn mount(
		&mut self,
		source: &str,
		target: &str,
		fstype: &str,
		flags: u32,
		data: Option<&str>,
	) -> Result<()> {
		// TODO: Implement proper mount logic
		// For now, just create a basic mount
		let sb = Arc::new(SuperBlock::new(fstype)?);
		let mount = Arc::new(VfsMount::new(sb, target, flags)?);
		self.mounts.push(mount);
		Ok(())
	}

	/// Allocate a new file descriptor
	pub fn alloc_fd(&mut self) -> i32 {
		let fd = self.next_fd;
		self.next_fd += 1;
		fd
	}

	/// Install a file into the file descriptor table
	pub fn install_fd(&mut self, fd: i32, file: Arc<File>) {
		self.fd_table.insert(fd, file);
	}

	/// Get a file by file descriptor
	pub fn get_file(&self, fd: i32) -> Option<Arc<File>> {
		self.fd_table.get(&fd).cloned()
	}

	/// Close a file descriptor
	pub fn close_fd(&mut self, fd: i32) -> Result<()> {
		self.fd_table.remove(&fd);
		Ok(())
	}
}

/// Initialize the VFS subsystem
pub fn init() -> Result<()> {
	// Register built-in filesystems
	ramfs::register_ramfs()?;
	procfs::register_procfs()?;
	devfs::register_devfs()?;

	// Create initial mounts
	mount::do_mount("none", "/", "ramfs", 0, None)?;

	// Create essential directories
	// TODO: Create /proc and /dev directories in root filesystem

	mount::do_mount("proc", "/proc", "proc", 0, None)?;
	mount::do_mount("devfs", "/dev", "devfs", 0, None)?;

	crate::console::print_info("VFS: Initialized virtual file system\n");
	Ok(())
}

/// Get a file descriptor from the table
pub fn get_file_descriptor(fd: i32) -> Option<Arc<File>> {
	let table = GLOBAL_FD_TABLE.lock();
	table.get(&fd).cloned()
}

/// Allocate a new file descriptor
pub fn allocate_file_descriptor(file: Arc<File>) -> Result<i32> {
	let fd = NEXT_FD.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
	let mut table = GLOBAL_FD_TABLE.lock();
	table.insert(fd, file);
	Ok(fd)
}

/// Close a file descriptor
pub fn close_file_descriptor(fd: i32) -> Result<()> {
	let mut table = GLOBAL_FD_TABLE.lock();
	table.remove(&fd).ok_or(Error::EBADF)?;
	Ok(())
}

/// Open a file
pub fn open_file(path: &str, flags: i32, mode: u32) -> Result<Arc<File>> {
	// For now, create a simple file structure
	// In a full implementation, this would:
	// 1. Parse the path
	// 2. Walk the directory tree
	// 3. Check permissions
	// 4. Create inode/dentry structures
	// 5. Return file handle

	let file = File::new(path, flags as u32, mode)?;

	Ok(Arc::new(file))
}

/// Read from a file
pub fn read_file(file: &Arc<File>, buf: &mut [u8]) -> Result<usize> {
	if let Some(ops) = &file.f_op {
		// Create a UserSlicePtr from the buffer for the interface
		let user_slice = unsafe { UserSlicePtr::new(buf.as_mut_ptr(), buf.len()) };
		let result = ops.read(file, user_slice, buf.len())?;
		Ok(result as usize)
	} else {
		Err(Error::ENOSYS)
	}
}

/// Write to a file
pub fn write_file(file: &Arc<File>, buf: &[u8]) -> Result<usize> {
	if let Some(ops) = &file.f_op {
		// Create a UserSlicePtr from the buffer for the interface
		let user_slice = unsafe { UserSlicePtr::new(buf.as_ptr() as *mut u8, buf.len()) };
		let result = ops.write(file, user_slice, buf.len())?;
		Ok(result as usize)
	} else {
		Err(Error::ENOSYS)
	}
}

/// Initialize VFS
pub fn init_vfs() -> Result<()> {
	// Initialize filesystems - just initialize the VFS, not individual filesystems
	crate::info!("VFS initialized");
	Ok(())
}

/// Open a file - Linux compatible sys_open
pub fn open(pathname: &str, flags: i32, mode: u32) -> Result<i32> {
	let mut vfs = VFS.lock();

	// TODO: Path resolution, permission checks, etc.
	// For now, create a simple file
	let file = Arc::new(File::new(pathname, flags as u32, mode)?);
	let fd = vfs.alloc_fd();
	vfs.install_fd(fd, file);

	Ok(fd)
}

/// Close a file descriptor - Linux compatible sys_close
pub fn close(fd: i32) -> Result<()> {
	let mut vfs = VFS.lock();
	vfs.close_fd(fd)
}

/// Read from a file descriptor - Linux compatible sys_read
pub fn read(fd: i32, buf: UserSlicePtr, count: usize) -> Result<isize> {
	let vfs = VFS.lock();
	let file = vfs.get_file(fd).ok_or(Error::EBADF)?;
	drop(vfs);

	file.read(buf, count)
}

/// Write to a file descriptor - Linux compatible sys_write
pub fn write(fd: i32, buf: UserSlicePtr, count: usize) -> Result<isize> {
	let vfs = VFS.lock();
	let file = vfs.get_file(fd).ok_or(Error::EBADF)?;
	drop(vfs);

	file.write(buf, count)
}

/// Seek within a file - Linux compatible sys_lseek
pub fn lseek(fd: i32, offset: i64, whence: i32) -> Result<i64> {
	let vfs = VFS.lock();
	let file = vfs.get_file(fd).ok_or(Error::EBADF)?;
	drop(vfs);

	file.seek(offset, whence)
}

/// Get file status - Linux compatible sys_fstat
pub fn fstat(fd: i32, statbuf: UserPtr<KStat>) -> Result<()> {
	let vfs = VFS.lock();
	let file = vfs.get_file(fd).ok_or(Error::EBADF)?;
	drop(vfs);

	let stat = file.stat()?;
	statbuf.write(stat)?;
	Ok(())
}

/// Generic file operations for simple filesystems
#[derive(Debug)]
pub struct GenericFileOps;

impl FileOperations for GenericFileOps {
	fn read(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Default read implementation
		Ok(0)
	}

	fn write(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		// Default write implementation
		Ok(count as isize)
	}

	fn seek(&self, file: &File, offset: i64, whence: i32) -> Result<i64> {
		// Default seek implementation
		match whence {
			SEEK_SET => Ok(offset),
			SEEK_CUR => Ok(
				file.pos.load(core::sync::atomic::Ordering::Relaxed) + offset
			),
			SEEK_END => Ok(offset), // TODO: Get file size
			_ => Err(Error::EINVAL),
		}
	}

	fn ioctl(&self, file: &File, cmd: u32, arg: usize) -> Result<isize> {
		Err(Error::ENOTTY)
	}

	fn mmap(&self, file: &File, vma: &mut crate::memory::VmaArea) -> Result<()> {
		Err(Error::ENODEV)
	}

	fn fsync(&self, file: &File, datasync: bool) -> Result<()> {
		Ok(())
	}

	fn poll(&self, file: &File, wait: &mut PollWait) -> Result<u32> {
		Ok(POLLIN | POLLOUT)
	}
}

/// Poll events
pub const POLLIN: u32 = 0x001;
pub const POLLPRI: u32 = 0x002;
pub const POLLOUT: u32 = 0x004;
pub const POLLERR: u32 = 0x008;
pub const POLLHUP: u32 = 0x010;
pub const POLLNVAL: u32 = 0x020;

/// Poll wait structure (simplified)
pub struct PollWait {
	// TODO: Implement proper poll/select mechanism
}

impl PollWait {
	pub fn new() -> Self {
		Self {}
	}
}

/// Global root filesystem
static ROOT_FS: Mutex<Option<Arc<SuperBlock>>> = Mutex::new(None);

/// Initialize root filesystem
pub fn init_root_fs() -> Result<()> {
	let ramfs_sb = ramfs::create_ramfs_superblock()?;
	*ROOT_FS.lock() = Some(ramfs_sb);
	Ok(())
}

/// Get root filesystem
pub fn get_root_fs() -> Result<Arc<SuperBlock>> {
	ROOT_FS.lock().clone().ok_or(Error::NotInitialized)
}
