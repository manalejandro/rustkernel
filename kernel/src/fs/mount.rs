// SPDX-License-Identifier: GPL-2.0

//! VFS mount abstraction - Linux compatible

use alloc::{format, string::String, vec::Vec}; // Add format macro
use core::sync::atomic::{AtomicU32, Ordering};

use crate::error::{Error, Result};
use crate::sync::{Arc, Mutex};

/// VFS mount structure - similar to Linux struct vfsmount
#[derive(Debug)]
pub struct VfsMount {
	/// Mounted superblock
	pub mnt_sb: Arc<super::SuperBlock>,
	/// Mount point path
	pub mnt_mountpoint: String,
	/// Mount flags
	pub mnt_flags: AtomicU32,
	/// Parent mount
	pub mnt_parent: Option<Arc<VfsMount>>,
	/// Child mounts
	pub mnt_children: Mutex<Vec<Arc<VfsMount>>>,
	/// Reference count
	pub mnt_count: AtomicU32,
	/// Device name
	pub mnt_devname: Option<String>,
	/// Mount options
	pub mnt_opts: Option<String>,
}

impl VfsMount {
	/// Create a new VFS mount
	pub fn new(sb: Arc<super::SuperBlock>, mountpoint: &str, flags: u32) -> Result<Self> {
		Ok(Self {
			mnt_sb: sb,
			mnt_mountpoint: String::from(mountpoint),
			mnt_flags: AtomicU32::new(flags),
			mnt_parent: None,
			mnt_children: Mutex::new(Vec::new()),
			mnt_count: AtomicU32::new(1),
			mnt_devname: None,
			mnt_opts: None,
		})
	}

	/// Set parent mount
	pub fn set_parent(&mut self, parent: Arc<VfsMount>) {
		self.mnt_parent = Some(parent);
	}

	/// Add child mount
	pub fn add_child(&self, child: Arc<VfsMount>) {
		let mut children = self.mnt_children.lock();
		children.push(child);
	}

	/// Get mount flags
	pub fn get_flags(&self) -> u32 {
		self.mnt_flags.load(Ordering::Relaxed)
	}

	/// Check if mount is read-only
	pub fn is_readonly(&self) -> bool {
		(self.get_flags() & super::super_block::MS_RDONLY) != 0
	}

	/// Check if mount is nosuid
	pub fn is_nosuid(&self) -> bool {
		(self.get_flags() & super::super_block::MS_NOSUID) != 0
	}

	/// Check if mount is nodev
	pub fn is_nodev(&self) -> bool {
		(self.get_flags() & super::super_block::MS_NODEV) != 0
	}

	/// Check if mount is noexec
	pub fn is_noexec(&self) -> bool {
		(self.get_flags() & super::super_block::MS_NOEXEC) != 0
	}

	/// Increment reference count
	pub fn mntget(&self) {
		self.mnt_count.fetch_add(1, Ordering::Relaxed);
	}

	/// Decrement reference count
	pub fn mntput(&self) {
		let old_count = self.mnt_count.fetch_sub(1, Ordering::Relaxed);
		if old_count == 1 {
			// Last reference, mount should be cleaned up
			// TODO: Unmount filesystem
		}
	}

	/// Get full mount path
	pub fn get_path(&self) -> String {
		if let Some(ref parent) = self.mnt_parent {
			if parent.mnt_mountpoint == "/" {
				self.mnt_mountpoint.clone()
			} else {
				format!("{}{}", parent.get_path(), self.mnt_mountpoint)
			}
		} else {
			self.mnt_mountpoint.clone()
		}
	}

	/// Find child mount by path
	pub fn find_child_mount(&self, path: &str) -> Option<Arc<VfsMount>> {
		let children = self.mnt_children.lock();
		for child in children.iter() {
			if child.mnt_mountpoint == path {
				return Some(child.clone());
			}
		}
		None
	}
}

unsafe impl Send for VfsMount {}
unsafe impl Sync for VfsMount {}

/// Mount namespace - similar to Linux struct mnt_namespace
pub struct MountNamespace {
	/// Root mount
	pub root: Option<Arc<VfsMount>>,
	/// All mounts in this namespace
	pub mounts: Mutex<Vec<Arc<VfsMount>>>,
	/// Namespace ID
	pub ns_id: u64,
	/// Reference count
	pub count: AtomicU32,
}

impl MountNamespace {
	/// Create a new mount namespace
	pub fn new(ns_id: u64) -> Self {
		Self {
			root: None,
			mounts: Mutex::new(Vec::new()),
			ns_id,
			count: AtomicU32::new(1),
		}
	}

	/// Add mount to namespace
	pub fn add_mount(&self, mount: Arc<VfsMount>) {
		let mut mounts = self.mounts.lock();
		mounts.push(mount);
	}

	/// Remove mount from namespace
	pub fn remove_mount(&self, mountpoint: &str) -> Option<Arc<VfsMount>> {
		let mut mounts = self.mounts.lock();
		if let Some(pos) = mounts.iter().position(|m| m.mnt_mountpoint == mountpoint) {
			Some(mounts.remove(pos))
		} else {
			None
		}
	}

	/// Find mount by path
	pub fn find_mount(&self, path: &str) -> Option<Arc<VfsMount>> {
		let mounts = self.mounts.lock();

		// Find the longest matching mount point
		let mut best_match: Option<Arc<VfsMount>> = None;
		let mut best_len = 0;

		for mount in mounts.iter() {
			let mount_path = mount.get_path();
			if path.starts_with(&mount_path) && mount_path.len() > best_len {
				best_match = Some(mount.clone());
				best_len = mount_path.len();
			}
		}

		best_match
	}

	/// Get all mount points
	pub fn get_mount_points(&self) -> Vec<String> {
		let mounts = self.mounts.lock();
		mounts.iter().map(|m| m.get_path()).collect()
	}

	/// Set root mount
	pub fn set_root(&mut self, root: Arc<VfsMount>) {
		self.root = Some(root.clone());
		self.add_mount(root);
	}
}

/// Global mount namespace
static INIT_MNT_NS: spin::once::Once<Mutex<MountNamespace>> = spin::once::Once::new();

fn get_init_mnt_ns() -> &'static Mutex<MountNamespace> {
	INIT_MNT_NS.call_once(|| Mutex::new(MountNamespace::new(1)))
}

/// Get the init mount namespace
pub fn get_init_ns() -> &'static Mutex<MountNamespace> {
	get_init_mnt_ns()
}

/// Mount a filesystem
pub fn do_mount(
	dev_name: &str,
	dir_name: &str,
	type_name: &str,
	flags: u32,
	data: Option<&str>,
) -> Result<()> {
	// TODO: Look up filesystem type
	// For now, create a basic mount
	let sb = Arc::new(super::SuperBlock::new(type_name)?);
	let mount = Arc::new(VfsMount::new(sb, dir_name, flags)?);

	let ns = get_init_ns();
	let ns = ns.lock();
	ns.add_mount(mount);

	crate::console::print_info(&format!(
		"Mounted {} on {} (type {})\n",
		dev_name, dir_name, type_name
	));
	Ok(())
}

/// Unmount a filesystem
pub fn do_umount(dir_name: &str, flags: u32) -> Result<()> {
	let ns = get_init_ns();
	let ns = ns.lock();

	if let Some(mount) = ns.remove_mount(dir_name) {
		mount.mntput();
		crate::console::print_info(&format!("Unmounted {}\n", dir_name));
		Ok(())
	} else {
		Err(Error::ENOENT)
	}
}

/// Get mount information for a path
pub fn path_get_mount(path: &str) -> Option<Arc<VfsMount>> {
	let ns = get_init_ns();
	let ns = ns.lock();
	ns.find_mount(path)
}

/// Check if a path is a mount point
pub fn is_mountpoint(path: &str) -> bool {
	let ns = get_init_ns();
	let ns = ns.lock();
	let mounts = ns.mounts.lock();
	mounts.iter().any(|m| m.get_path() == path)
}

/// Get all mount points
pub fn get_all_mounts() -> Vec<String> {
	let ns = get_init_ns();
	let ns = ns.lock();
	ns.get_mount_points()
}

/// Remount a filesystem with new flags
pub fn do_remount(dir_name: &str, flags: u32, data: Option<&str>) -> Result<()> {
	let ns = get_init_ns();
	let ns = ns.lock();

	if let Some(mount) = ns.find_mount(dir_name) {
		mount.mnt_flags.store(flags, Ordering::Relaxed);

		// Also remount the superblock
		if let Some(ref ops) = mount.mnt_sb.s_op {
			ops.remount_fs(&mount.mnt_sb, flags, data)?;
		}

		crate::console::print_info(&format!(
			"Remounted {} with flags {:#x}\n",
			dir_name, flags
		));
		Ok(())
	} else {
		Err(Error::ENOENT)
	}
}

/// Bind mount - create a bind mount
pub fn do_bind_mount(old_path: &str, new_path: &str, flags: u32) -> Result<()> {
	let ns = get_init_ns();
	let ns = ns.lock();

	if let Some(old_mount) = ns.find_mount(old_path) {
		let new_mount = Arc::new(VfsMount::new(
			old_mount.mnt_sb.clone(),
			new_path,
			flags | super::super_block::MS_BIND,
		)?);
		ns.add_mount(new_mount);

		crate::console::print_info(&format!("Bind mounted {} to {}\n", old_path, new_path));
		Ok(())
	} else {
		Err(Error::ENOENT)
	}
}
