// SPDX-License-Identifier: GPL-2.0

//! Directory entry (dentry) abstraction - Linux compatible

use alloc::{format, string::String, vec::Vec}; // Add format macro
use core::sync::atomic::{AtomicU32, Ordering};

use crate::error::Result;
use crate::sync::{Arc, Mutex};

/// Dentry structure - similar to Linux struct dentry
#[derive(Debug)]
pub struct Dentry {
	/// Entry name
	pub d_name: String,
	/// Associated inode
	pub d_inode: Option<Arc<super::Inode>>,
	/// Parent dentry
	pub d_parent: Option<Arc<Dentry>>,
	/// Child entries (for directories)
	pub d_subdirs: Mutex<Vec<Arc<Dentry>>>,
	/// Dentry operations
	pub d_op: Option<Arc<dyn DentryOperations>>,
	/// Superblock
	pub d_sb: Option<Arc<super::SuperBlock>>,
	/// Reference count
	pub d_count: AtomicU32,
	/// Dentry flags
	pub d_flags: AtomicU32,
	/// Hash for dcache
	pub d_hash: u32,
}

impl Dentry {
	/// Create a new dentry
	pub fn new(name: String, inode: Option<Arc<super::Inode>>) -> Self {
		Self {
			d_name: name,
			d_inode: inode,
			d_parent: None,
			d_subdirs: Mutex::new(Vec::new()),
			d_op: None,
			d_sb: None,
			d_count: AtomicU32::new(1),
			d_flags: AtomicU32::new(0),
			d_hash: 0, // TODO: Calculate hash
		}
	}

	/// Set parent dentry
	pub fn set_parent(&mut self, parent: Arc<Dentry>) {
		self.d_parent = Some(parent);
	}

	/// Add child dentry
	pub fn add_child(&self, child: Arc<Dentry>) {
		let mut subdirs = self.d_subdirs.lock();
		subdirs.push(child);
	}

	/// Find child dentry by name
	pub fn find_child(&self, name: &str) -> Option<Arc<Dentry>> {
		let subdirs = self.d_subdirs.lock();
		for child in subdirs.iter() {
			if child.d_name == name {
				return Some(child.clone());
			}
		}
		None
	}

	/// Get full path of this dentry
	pub fn get_path(&self) -> String {
		if let Some(ref parent) = self.d_parent {
			if parent.d_name == "/" {
				format!("/{}", self.d_name)
			} else {
				format!("{}/{}", parent.get_path(), self.d_name)
			}
		} else {
			self.d_name.clone()
		}
	}

	/// Check if dentry is root
	pub fn is_root(&self) -> bool {
		self.d_parent.is_none() || self.d_name == "/"
	}

	/// Increment reference count
	pub fn dget(&self) {
		self.d_count.fetch_add(1, Ordering::Relaxed);
	}

	/// Decrement reference count
	pub fn dput(&self) {
		let old_count = self.d_count.fetch_sub(1, Ordering::Relaxed);
		if old_count == 1 {
			// Last reference, dentry should be cleaned up
			// TODO: Call d_delete operation if present
		}
	}

	/// Revalidate dentry
	pub fn revalidate(&self) -> Result<bool> {
		if let Some(ref ops) = self.d_op {
			ops.revalidate(self)
		} else {
			Ok(true) // Always valid by default
		}
	}

	/// Delete dentry
	pub fn delete(&self) -> Result<()> {
		if let Some(ref ops) = self.d_op {
			ops.delete(self)
		} else {
			Ok(())
		}
	}

	/// Compare two dentries
	pub fn compare(&self, other: &Dentry) -> Result<bool> {
		if let Some(ref ops) = self.d_op {
			ops.compare(self, other)
		} else {
			Ok(self.d_name == other.d_name)
		}
	}

	/// Hash dentry name
	pub fn hash(&self) -> Result<u32> {
		if let Some(ref ops) = self.d_op {
			ops.hash(self)
		} else {
			// Simple hash function
			let mut hash = 0u32;
			for byte in self.d_name.bytes() {
				hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
			}
			Ok(hash)
		}
	}
}

unsafe impl Send for Dentry {}
unsafe impl Sync for Dentry {}

/// Dentry operations trait - similar to Linux dentry_operations
pub trait DentryOperations: Send + Sync + core::fmt::Debug {
	/// Revalidate dentry
	fn revalidate(&self, dentry: &Dentry) -> Result<bool>;

	/// Hash dentry name
	fn hash(&self, dentry: &Dentry) -> Result<u32>;

	/// Compare two dentries
	fn compare(&self, d1: &Dentry, d2: &Dentry) -> Result<bool>;

	/// Delete dentry
	fn delete(&self, dentry: &Dentry) -> Result<()>;

	/// Release dentry
	fn release(&self, dentry: &Dentry) -> Result<()>;

	/// Canonicalize path
	fn canonical_path(&self, dentry: &Dentry) -> Result<String>;
}

/// Generic dentry operations
#[derive(Debug)]
pub struct GenericDentryOps;

impl DentryOperations for GenericDentryOps {
	fn revalidate(&self, dentry: &Dentry) -> Result<bool> {
		Ok(true)
	}

	fn hash(&self, dentry: &Dentry) -> Result<u32> {
		dentry.hash()
	}

	fn compare(&self, d1: &Dentry, d2: &Dentry) -> Result<bool> {
		Ok(d1.d_name == d2.d_name)
	}

	fn delete(&self, dentry: &Dentry) -> Result<()> {
		Ok(())
	}

	fn release(&self, dentry: &Dentry) -> Result<()> {
		Ok(())
	}

	fn canonical_path(&self, dentry: &Dentry) -> Result<String> {
		Ok(dentry.get_path())
	}
}

/// Dentry cache (dcache) - simplified version
pub struct DentryCache {
	/// Cached dentries
	cache: Mutex<alloc::collections::BTreeMap<String, Arc<Dentry>>>,
	/// Hash buckets for faster lookup
	hash_table: Vec<Mutex<Vec<Arc<Dentry>>>>,
}

impl DentryCache {
	/// Create a new dentry cache
	pub fn new() -> Self {
		const HASH_BUCKETS: usize = 256;
		let mut hash_table = Vec::with_capacity(HASH_BUCKETS);
		for _ in 0..HASH_BUCKETS {
			hash_table.push(Mutex::new(Vec::new()));
		}

		Self {
			cache: Mutex::new(alloc::collections::BTreeMap::new()),
			hash_table,
		}
	}

	/// Look up dentry by path
	pub fn lookup(&self, path: &str) -> Option<Arc<Dentry>> {
		let cache = self.cache.lock();
		cache.get(path).cloned()
	}

	/// Insert dentry into cache
	pub fn insert(&self, path: String, dentry: Arc<Dentry>) {
		let mut cache = self.cache.lock();
		cache.insert(path, dentry.clone());

		// Also insert into hash table
		let hash = dentry.d_hash as usize % self.hash_table.len();
		let mut bucket = self.hash_table[hash].lock();
		bucket.push(dentry);
	}

	/// Remove dentry from cache
	pub fn remove(&self, path: &str) -> Option<Arc<Dentry>> {
		let mut cache = self.cache.lock();
		cache.remove(path)
	}

	/// Prune unused dentries
	pub fn prune(&self) {
		let mut cache = self.cache.lock();
		cache.retain(|_, dentry| dentry.d_count.load(Ordering::Relaxed) > 1);

		// Also prune hash table
		for bucket in &self.hash_table {
			let mut bucket = bucket.lock();
			bucket.retain(|dentry| dentry.d_count.load(Ordering::Relaxed) > 1);
		}
	}
}

/// Global dentry cache
static DCACHE: spin::once::Once<DentryCache> = spin::once::Once::new();

fn get_dcache() -> &'static DentryCache {
	DCACHE.call_once(|| DentryCache::new())
}

/// Look up dentry in cache
pub fn dcache_lookup(path: &str) -> Option<Arc<Dentry>> {
	get_dcache().lookup(path)
}

/// Insert dentry into cache
pub fn dcache_insert(path: String, dentry: Arc<Dentry>) {
	get_dcache().insert(path, dentry);
}

/// Remove dentry from cache
pub fn dcache_remove(path: &str) -> Option<Arc<Dentry>> {
	get_dcache().remove(path)
}

/// Prune dentry cache
pub fn dcache_prune() {
	get_dcache().prune();
}
