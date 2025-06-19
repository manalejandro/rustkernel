// SPDX-License-Identifier: GPL-2.0

//! Proc filesystem implementation - Linux compatible

use alloc::{boxed::Box, collections::BTreeMap, format, string::String, vec, vec::Vec}; /* Add vec macro and Box */
use core::sync::atomic::{AtomicU64, Ordering};

use crate::error::{Error, Result};
use crate::fs::*;
use crate::memory::UserSlicePtr;
use crate::sync::{Arc, Mutex};

/// Proc filesystem entry
#[derive(Debug)]
pub struct ProcEntry {
	/// Entry name
	pub name: String,
	/// Entry type
	pub entry_type: ProcEntryType,
	/// Mode
	pub mode: u32,
	/// Read function
	pub read: Option<fn(&ProcEntry, &mut String) -> Result<()>>,
	/// Write function
	pub write: Option<fn(&ProcEntry, &str) -> Result<()>>,
	/// Child entries (for directories)
	pub children: Mutex<Vec<Arc<ProcEntry>>>,
	/// Parent entry
	pub parent: Option<Arc<ProcEntry>>,
	/// Private data
	pub private_data: Option<*mut u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProcEntryType {
	File,
	Directory,
	Symlink,
}

impl ProcEntry {
	pub fn new_file(
		name: String,
		mode: u32,
		read_fn: fn(&ProcEntry, &mut String) -> Result<()>,
	) -> Self {
		Self {
			name,
			entry_type: ProcEntryType::File,
			mode,
			read: Some(read_fn),
			write: None,
			children: Mutex::new(Vec::new()),
			parent: None,
			private_data: None,
		}
	}

	pub fn new_dir(name: String, mode: u32) -> Self {
		Self {
			name,
			entry_type: ProcEntryType::Directory,
			mode,
			read: None,
			write: None,
			children: Mutex::new(Vec::new()),
			parent: None,
			private_data: None,
		}
	}

	pub fn add_child(self: &Arc<Self>, child: Arc<ProcEntry>) {
		let mut children = self.children.lock();
		children.push(child);
	}

	pub fn find_child(&self, name: &str) -> Option<Arc<ProcEntry>> {
		let children = self.children.lock();
		for child in children.iter() {
			if child.name == name {
				return Some(child.clone());
			}
		}
		None
	}
}

unsafe impl Send for ProcEntry {}
unsafe impl Sync for ProcEntry {}

/// Proc filesystem
pub struct ProcFs {
	/// Root entry
	pub root: Arc<ProcEntry>,
	/// Next inode number
	next_ino: AtomicU64,
	/// Entry to inode mapping
	entries: Mutex<BTreeMap<*const ProcEntry, u64>>,
}

impl ProcFs {
	pub fn new() -> Self {
		let root = Arc::new(ProcEntry::new_dir(String::from("proc"), 0o755));
		let mut fs = Self {
			root,
			next_ino: AtomicU64::new(1),
			entries: Mutex::new(BTreeMap::new()),
		};

		fs.create_default_entries();
		fs
	}

	fn alloc_ino(&self) -> u64 {
		self.next_ino.fetch_add(1, Ordering::Relaxed)
	}

	fn get_or_create_ino(&self, entry: &ProcEntry) -> u64 {
		let entry_ptr = entry as *const ProcEntry;
		let mut entries = self.entries.lock();

		if let Some(&ino) = entries.get(&entry_ptr) {
			ino
		} else {
			let ino = self.alloc_ino();
			entries.insert(entry_ptr, ino);
			ino
		}
	}

	fn create_default_entries(&mut self) {
		// Create /proc/version
		let version_entry = Arc::new(ProcEntry::new_file(
			String::from("version"),
			0o444,
			proc_version_read,
		));
		self.root.add_child(version_entry);

		// Create /proc/meminfo
		let meminfo_entry = Arc::new(ProcEntry::new_file(
			String::from("meminfo"),
			0o444,
			proc_meminfo_read,
		));
		self.root.add_child(meminfo_entry);

		// Create /proc/cpuinfo
		let cpuinfo_entry = Arc::new(ProcEntry::new_file(
			String::from("cpuinfo"),
			0o444,
			proc_cpuinfo_read,
		));
		self.root.add_child(cpuinfo_entry);

		// Create /proc/uptime
		let uptime_entry = Arc::new(ProcEntry::new_file(
			String::from("uptime"),
			0o444,
			proc_uptime_read,
		));
		self.root.add_child(uptime_entry);

		// Create /proc/loadavg
		let loadavg_entry = Arc::new(ProcEntry::new_file(
			String::from("loadavg"),
			0o444,
			proc_loadavg_read,
		));
		self.root.add_child(loadavg_entry);

		// Create /proc/stat
		let stat_entry = Arc::new(ProcEntry::new_file(
			String::from("stat"),
			0o444,
			proc_stat_read,
		));
		self.root.add_child(stat_entry);

		// Create /proc/mounts
		let mounts_entry = Arc::new(ProcEntry::new_file(
			String::from("mounts"),
			0o444,
			proc_mounts_read,
		));
		self.root.add_child(mounts_entry);
	}
}

/// Proc filesystem file operations
#[derive(Debug)]
pub struct ProcFileOps {
	entry: Arc<ProcEntry>,
}

impl ProcFileOps {
	pub fn new(entry: Arc<ProcEntry>) -> Self {
		Self { entry }
	}
}

impl FileOperations for ProcFileOps {
	fn read(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		match self.entry.entry_type {
			ProcEntryType::File => {
				if let Some(read_fn) = self.entry.read {
					let mut content = String::new();
					read_fn(&self.entry, &mut content)?;

					let pos = file.get_pos() as usize;
					if pos >= content.len() {
						return Ok(0);
					}

					let to_copy = core::cmp::min(count, content.len() - pos);
					let data = &content.as_bytes()[pos..pos + to_copy];

					buf.copy_from_slice(data)?;
					file.set_pos(file.get_pos() + to_copy as i64);

					Ok(to_copy as isize)
				} else {
					Ok(0)
				}
			}
			_ => Err(Error::EISDIR),
		}
	}

	fn write(&self, file: &File, buf: UserSlicePtr, count: usize) -> Result<isize> {
		if let Some(write_fn) = self.entry.write {
			let mut data = vec![0u8; count];
			buf.copy_to_slice(&mut data)?;
			let content = String::from_utf8(data).map_err(|_| Error::EINVAL)?;
			write_fn(&self.entry, &content)?;
			Ok(count as isize)
		} else {
			Err(Error::EPERM)
		}
	}

	fn seek(&self, file: &File, offset: i64, whence: i32) -> Result<i64> {
		let new_pos = match whence {
			SEEK_SET => offset,
			SEEK_CUR => file.get_pos() + offset,
			SEEK_END => 0, // Proc files are typically small
			_ => return Err(Error::EINVAL),
		};

		if new_pos < 0 {
			return Err(Error::EINVAL);
		}

		file.set_pos(new_pos);
		Ok(new_pos)
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

/// Proc filesystem inode operations
#[derive(Debug)]
pub struct ProcInodeOps {
	fs: *const ProcFs,
}

impl ProcInodeOps {
	pub fn new(fs: &ProcFs) -> Self {
		Self {
			fs: fs as *const ProcFs,
		}
	}

	fn get_fs(&self) -> &ProcFs {
		unsafe { &*self.fs }
	}
}

unsafe impl Send for ProcInodeOps {}
unsafe impl Sync for ProcInodeOps {}

impl InodeOperations for ProcInodeOps {
	fn lookup(&self, dir: &Inode, name: &str) -> Result<Arc<Inode>> {
		let fs = self.get_fs();
		// Find the proc entry for this inode
		// This is a simplified implementation
		if let Some(child) = fs.root.find_child(name) {
			let ino = fs.get_or_create_ino(&child);
			let mode = match child.entry_type {
				ProcEntryType::File => mode::S_IFREG | child.mode,
				ProcEntryType::Directory => mode::S_IFDIR | child.mode,
				ProcEntryType::Symlink => mode::S_IFLNK | child.mode,
			};

			let mut inode = Inode::new(ino, mode);
			inode.set_operations(Arc::new(ProcInodeOps::new(fs)));
			inode.set_file_operations(Arc::new(ProcFileOps::new(child)));

			Ok(Arc::new(inode))
		} else {
			Err(Error::ENOENT)
		}
	}

	fn create(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>> {
		Err(Error::EPERM) // Proc filesystem is read-only
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

	fn getxattr(&self, inode: &Inode, name: &str) -> Result<Vec<u8>> {
		Err(Error::ENODATA)
	}

	fn setxattr(&self, inode: &Inode, name: &str, value: &[u8], flags: u32) -> Result<()> {
		Err(Error::EPERM)
	}

	fn listxattr(&self, inode: &Inode) -> Result<Vec<String>> {
		Ok(Vec::new())
	}

	fn removexattr(&self, inode: &Inode, name: &str) -> Result<()> {
		Err(Error::EPERM)
	}
}

// Proc file read functions

fn proc_version_read(_entry: &ProcEntry, content: &mut String) -> Result<()> {
	content.push_str(&format!(
		"{} version {} ({})\n",
		crate::NAME,
		crate::VERSION,
		"rustc"
	));
	Ok(())
}

fn proc_meminfo_read(_entry: &ProcEntry, content: &mut String) -> Result<()> {
	// TODO: Get actual memory statistics
	let total_mem = 128 * 1024; // 128 MB placeholder
	let free_mem = 64 * 1024; // 64 MB placeholder

	content.push_str(&format!(
		"MemTotal:     {} kB\n\
         MemFree:      {} kB\n\
         MemAvailable: {} kB\n\
         Buffers:      {} kB\n\
         Cached:       {} kB\n",
		total_mem, free_mem, free_mem, 0, 0
	));
	Ok(())
}

fn proc_cpuinfo_read(_entry: &ProcEntry, content: &mut String) -> Result<()> {
	// TODO: Get actual CPU information
	content.push_str(
		"processor\t: 0\n\
         vendor_id\t: RustKernel\n\
         cpu family\t: 1\n\
         model\t\t: 1\n\
         model name\t: Rust Kernel CPU\n\
         stepping\t: 1\n\
         microcode\t: 0x1\n\
         cpu MHz\t\t: 1000.000\n\
         cache size\t: 1024 KB\n\
         flags\t\t: rust\n\n",
	);
	Ok(())
}

fn proc_uptime_read(_entry: &ProcEntry, content: &mut String) -> Result<()> {
	// TODO: Get actual uptime
	let uptime = 100.0; // 100 seconds placeholder
	let idle = 90.0; // 90 seconds idle placeholder

	content.push_str(&format!("{:.2} {:.2}\n", uptime, idle));
	Ok(())
}

fn proc_loadavg_read(_entry: &ProcEntry, content: &mut String) -> Result<()> {
	// TODO: Get actual load average
	content.push_str("0.00 0.00 0.00 1/1 1\n");
	Ok(())
}

fn proc_stat_read(_entry: &ProcEntry, content: &mut String) -> Result<()> {
	// TODO: Get actual system statistics
	content.push_str(
		"cpu  0 0 0 1000 0 0 0 0 0 0\n\
         cpu0 0 0 0 1000 0 0 0 0 0 0\n\
         intr 0\n\
         ctxt 0\n\
         btime 0\n\
         processes 1\n\
         procs_running 1\n\
         procs_blocked 0\n",
	);
	Ok(())
}

fn proc_mounts_read(_entry: &ProcEntry, content: &mut String) -> Result<()> {
	// TODO: Get actual mount information
	let mounts = crate::fs::mount::get_all_mounts();
	for mount in mounts {
		content.push_str(&format!("none {} ramfs rw 0 0\n", mount));
	}
	Ok(())
}

/// Mount proc filesystem
pub fn mount_procfs(_dev_name: &str, _flags: u32, _data: Option<&str>) -> Result<Arc<SuperBlock>> {
	let mut sb = SuperBlock::new("proc")?;
	sb.s_magic = 0x9fa0; // PROC_SUPER_MAGIC

	let procfs = Box::leak(Box::new(ProcFs::new()));
	sb.s_fs_info = Some(procfs as *mut ProcFs as *mut u8);

	// Create root inode
	let root_inode = Arc::new({
		let mut inode = Inode::new(1, mode::S_IFDIR | 0o755);
		inode.set_operations(Arc::new(ProcInodeOps::new(procfs)));
		inode
	});

	let root_dentry = Arc::new(Dentry::new(String::from("/"), Some(root_inode)));
	sb.s_root = Some(root_dentry);

	Ok(Arc::new(sb))
}

/// Register proc filesystem
pub fn register_procfs() -> Result<()> {
	let procfs_type = FileSystemType::new(
		String::from("proc"),
		|_fstype, flags, _dev_name, data| mount_procfs(_dev_name, flags, data),
		|_sb| Ok(()),
	);

	// TODO: Register with VFS
	crate::console::print_info("Registered proc filesystem\n");
	Ok(())
}
