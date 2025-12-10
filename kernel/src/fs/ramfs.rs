// SPDX-License-Identifier: GPL-2.0

//! Simple RAM filesystem implementation

use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec}; // Add Box import
use core::sync::atomic::{AtomicU64, Ordering};

use crate::error::{Error, Result};
use crate::fs::inode::GenericInodeOps;
use crate::fs::*;
use crate::sync::{Arc, Mutex};

const NAME_MAX: usize = 255;

/// RAM filesystem superblock
pub struct RamFs {
	/// Next inode number
	next_ino: AtomicU64,
	/// Inode storage
	inodes: Mutex<BTreeMap<u64, Arc<Inode>>>,
	/// Directory entries
	entries: Mutex<BTreeMap<u64, Vec<Arc<Dentry>>>>,
}

impl RamFs {
	pub fn new() -> Self {
		Self {
			next_ino: AtomicU64::new(1),
			inodes: Mutex::new(BTreeMap::new()),
			entries: Mutex::new(BTreeMap::new()),
		}
	}

	fn alloc_ino(&self) -> u64 {
		self.next_ino.fetch_add(1, Ordering::Relaxed)
	}

	fn create_inode(&self, mode: u32) -> Arc<Inode> {
		let ino = self.alloc_ino();
		let mut inode = Inode::new(ino, mode);
		inode.set_operations(Arc::new(RamFsInodeOps::new(self)));

		let inode = Arc::new(inode);

		let mut inodes = self.inodes.lock();
		inodes.insert(ino, inode.clone());

		if mode::s_isdir(mode) {
			let mut entries = self.entries.lock();
			entries.insert(ino, Vec::new());
		}

		inode
	}

	fn get_inode(&self, ino: u64) -> Option<Arc<Inode>> {
		let inodes = self.inodes.lock();
		inodes.get(&ino).cloned()
	}

	fn add_entry(&self, dir_ino: u64, name: String, child_ino: u64) -> Result<()> {
		let child_inode = self.get_inode(child_ino).ok_or(Error::ENOENT)?;
		let dentry = Arc::new(Dentry::new(name, Some(child_inode)));

		let mut entries = self.entries.lock();
		if let Some(dir_entries) = entries.get_mut(&dir_ino) {
			dir_entries.push(dentry);
			Ok(())
		} else {
			Err(Error::ENOTDIR)
		}
	}

	fn find_entry(&self, dir_ino: u64, name: &str) -> Option<Arc<Dentry>> {
		let entries = self.entries.lock();
		if let Some(dir_entries) = entries.get(&dir_ino) {
			for entry in dir_entries {
				if entry.d_name == name {
					return Some(entry.clone());
				}
			}
		}
		None
	}

	fn remove_entry(&self, dir_ino: u64, name: &str) -> Result<()> {
		let mut entries = self.entries.lock();
		if let Some(dir_entries) = entries.get_mut(&dir_ino) {
			if let Some(pos) = dir_entries.iter().position(|e| e.d_name == name) {
				dir_entries.remove(pos);
				Ok(())
			} else {
				Err(Error::ENOENT)
			}
		} else {
			Err(Error::ENOTDIR)
		}
	}
}

/// RAM filesystem inode operations
#[derive(Debug)]
pub struct RamFsInodeOps {
	fs: *const RamFs,
}

impl RamFsInodeOps {
	fn new(fs: &RamFs) -> Self {
		Self {
			fs: fs as *const RamFs,
		}
	}

	fn get_fs(&self) -> &RamFs {
		unsafe { &*self.fs }
	}
}

unsafe impl Send for RamFsInodeOps {}
unsafe impl Sync for RamFsInodeOps {}

impl InodeOperations for RamFsInodeOps {
	fn lookup(&self, dir: &Inode, name: &str) -> Result<Arc<Inode>> {
		let fs = self.get_fs();
		if let Some(entry) = fs.find_entry(dir.i_ino, name) {
			if let Some(inode) = &entry.d_inode {
				Ok(Arc::clone(inode))
			} else {
				Err(Error::ENOENT)
			}
		} else {
			Err(Error::ENOENT)
		}
	}

	fn create(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>> {
		let fs = self.get_fs();

		// Check if entry already exists
		if fs.find_entry(dir.i_ino, name).is_some() {
			return Err(Error::EEXIST);
		}

		let inode = fs.create_inode(mode | mode::S_IFREG);
		fs.add_entry(dir.i_ino, String::from(name), inode.i_ino)?;

		Ok(inode)
	}

	fn mkdir(&self, dir: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>> {
		let fs = self.get_fs();

		// Check if entry already exists
		if fs.find_entry(dir.i_ino, name).is_some() {
			return Err(Error::EEXIST);
		}

		let inode = fs.create_inode(mode | mode::S_IFDIR);
		fs.add_entry(dir.i_ino, String::from(name), inode.i_ino)?;

		// Add . and .. entries
		fs.add_entry(inode.i_ino, String::from("."), inode.i_ino)?;
		fs.add_entry(inode.i_ino, String::from(".."), dir.i_ino)?;

		Ok(inode)
	}

	fn unlink(&self, dir: &Inode, name: &str) -> Result<()> {
		let fs = self.get_fs();
		fs.remove_entry(dir.i_ino, name)
	}

	fn rmdir(&self, dir: &Inode, name: &str) -> Result<()> {
		let fs = self.get_fs();

		// Check if directory is empty (only . and .. entries)
		let entries = fs.entries.lock();
		if let Some(target_inode) = fs
			.find_entry(dir.i_ino, name)
			.and_then(|e| e.d_inode.clone())
		{
			if let Some(dir_entries) = entries.get(&target_inode.i_ino) {
				if dir_entries.len() > 2 {
					return Err(Error::ENOTEMPTY);
				}
			}
		}
		drop(entries);

		fs.remove_entry(dir.i_ino, name)
	}

	fn symlink(&self, dir: &Inode, name: &str, target: &str) -> Result<Arc<Inode>> {
		let fs = self.get_fs();

		// Check if entry already exists
		if fs.find_entry(dir.i_ino, name).is_some() {
			return Err(Error::EEXIST);
		}

		let inode = fs.create_inode(mode::S_IFLNK | 0o777);
		// TODO: Store symlink target
		fs.add_entry(dir.i_ino, String::from(name), inode.i_ino)?;

		Ok(inode)
	}

	fn rename(
		&self,
		old_dir: &Inode,
		old_name: &str,
		new_dir: &Inode,
		new_name: &str,
	) -> Result<()> {
		let fs = self.get_fs();

		// Find the entry to rename
		if let Some(entry) = fs.find_entry(old_dir.i_ino, old_name) {
			// Remove from old location
			fs.remove_entry(old_dir.i_ino, old_name)?;

			// Add to new location
			if let Some(inode) = &entry.d_inode {
				fs.add_entry(new_dir.i_ino, String::from(new_name), inode.i_ino)?;
			}

			Ok(())
		} else {
			Err(Error::ENOENT)
		}
	}

	fn setattr(&self, inode: &Inode, attr: &InodeAttr) -> Result<()> {
		// Apply basic attributes using generic implementation
		let generic_ops = GenericInodeOps;
		generic_ops.setattr(inode, attr)
	}

	fn getattr(&self, inode: &Inode) -> Result<InodeAttr> {
		let generic_ops = GenericInodeOps;
		generic_ops.getattr(inode)
	}

	fn readlink(&self, inode: &Inode) -> Result<String> {
		// TODO: Return stored symlink target
		Err(Error::EINVAL)
	}

	fn follow_link(&self, inode: &Inode) -> Result<Arc<Inode>> {
		// TODO: Follow symlink to target
		Err(Error::EINVAL)
	}

	fn truncate(&self, inode: &Inode, size: u64) -> Result<()> {
		inode.set_size(size);
		Ok(())
	}

	fn getxattr(&self, inode: &Inode, name: &str) -> Result<Vec<u8>> {
		Err(Error::ENODATA)
	}

	fn setxattr(&self, inode: &Inode, name: &str, value: &[u8], flags: u32) -> Result<()> {
		Err(Error::ENOSYS)
	}

	fn listxattr(&self, inode: &Inode) -> Result<Vec<String>> {
		Ok(Vec::new())
	}

	fn removexattr(&self, inode: &Inode, name: &str) -> Result<()> {
		Err(Error::ENODATA)
	}
}

/// Mount RAM filesystem
pub fn mount_ramfs(_dev_name: &str, _flags: u32, _data: Option<&str>) -> Result<Arc<SuperBlock>> {
	let mut sb = SuperBlock::new("ramfs")?;
	sb.s_magic = 0x858458f6; // RAMFS magic
	sb.set_operations(Arc::new(RamFsSuperOps));

	let ramfs = Box::leak(Box::new(RamFs::new()));
	sb.s_fs_info = Some(ramfs as *mut RamFs as *mut u8);

	// Create root directory
	let root_inode = ramfs.create_inode(mode::S_IFDIR | 0o755);
	let root_dentry = Arc::new(Dentry::new(String::from("/"), Some(root_inode)));

	let sb = Arc::new(sb);
	Ok(sb)
}

/// RAM filesystem superblock operations
#[derive(Debug)]
pub struct RamFsSuperOps;

impl SuperOperations for RamFsSuperOps {
	fn alloc_inode(&self, sb: &SuperBlock) -> Result<Arc<Inode>> {
		let ramfs = unsafe { &*(sb.s_fs_info.unwrap() as *const RamFs) };
		Ok(ramfs.create_inode(0o644))
	}

	fn destroy_inode(&self, inode: &Inode) -> Result<()> {
		Ok(())
	}

	fn write_inode(&self, inode: &Inode, sync: bool) -> Result<()> {
		// RAM filesystem doesn't need to write inodes
		Ok(())
	}

	fn delete_inode(&self, inode: &Inode) -> Result<()> {
		Ok(())
	}

	fn put_super(&self, sb: &SuperBlock) -> Result<()> {
		if let Some(fs_info) = sb.s_fs_info {
			unsafe {
				let ramfs = Box::from_raw(fs_info as *mut RamFs);
				drop(ramfs);
			}
		}
		Ok(())
	}

	fn write_super(&self, sb: &SuperBlock) -> Result<()> {
		// Nothing to write for RAM filesystem
		Ok(())
	}

	fn sync_fs(&self, sb: &SuperBlock, wait: bool) -> Result<()> {
		// Nothing to sync for RAM filesystem
		Ok(())
	}

	fn freeze_fs(&self, sb: &SuperBlock) -> Result<()> {
		Ok(())
	}

	fn unfreeze_fs(&self, sb: &SuperBlock) -> Result<()> {
		Ok(())
	}

	fn statfs(&self, sb: &SuperBlock) -> Result<KStatFs> {
		Ok(KStatFs {
			f_type: sb.s_magic as u64,
			f_bsize: sb.s_blocksize as u64,
			f_blocks: 0, // Unlimited
			f_bfree: 0,  // Unlimited
			f_bavail: 0, // Unlimited
			f_files: 0,  // Dynamic
			f_ffree: 0,  // Unlimited
			f_fsid: [0, 0],
			f_namelen: NAME_MAX as u64,
			f_frsize: sb.s_blocksize as u64,
			f_flags: 0,
			f_spare: [0; 4],
		})
	}

	fn remount_fs(&self, sb: &SuperBlock, flags: u32, data: Option<&str>) -> Result<()> {
		sb.s_flags.store(flags, Ordering::Relaxed);
		Ok(())
	}

	fn show_options(&self, sb: &SuperBlock) -> Result<String> {
		Ok(String::new())
	}
}

/// Kill RAM filesystem superblock
pub fn kill_ramfs(sb: &SuperBlock) -> Result<()> {
	if let Some(ref ops) = sb.s_op {
		ops.put_super(sb)
	} else {
		Ok(())
	}
}

/// Create a RAM filesystem superblock
pub fn create_ramfs_superblock() -> Result<Arc<SuperBlock>> {
	let mut sb = SuperBlock::new("ramfs")?;
	sb.s_magic = 0x858458f6; // RAMFS magic
	sb.set_operations(Arc::new(RamFsSuperOps));

	let ramfs = Box::leak(Box::new(RamFs::new()));
	sb.s_fs_info = Some(ramfs as *mut RamFs as *mut u8);

	// Create root directory
	let root_inode = ramfs.create_inode(mode::S_IFDIR | 0o755);

	Ok(Arc::new(sb))
}

/// Register RAM filesystem
pub fn register_ramfs() -> Result<()> {
	let ramfs_type = FileSystemType::new(
		String::from("ramfs"),
		|_fstype, flags, _dev_name, data| mount_ramfs(_dev_name, flags, data),
		|sb| kill_ramfs(sb),
	);

	// TODO: Register with VFS
	crate::console::print_info("Registered ramfs filesystem\n");
	Ok(())
}
