// SPDX-License-Identifier: GPL-2.0

//! Path resolution and manipulation - Linux compatible

use alloc::{
	format,
	string::{String, ToString},
	vec::Vec,
};

use crate::error::{Error, Result};
use crate::sync::Arc; // Add format macro and ToString

/// Path structure for path resolution
#[derive(Debug, Clone)]
pub struct Path {
	/// Mount point
	pub mnt: Option<Arc<super::VfsMount>>,
	/// Dentry
	pub dentry: Option<Arc<super::Dentry>>,
}

impl Path {
	/// Create a new path
	pub fn new() -> Self {
		Self {
			mnt: None,
			dentry: None,
		}
	}

	/// Create path from mount and dentry
	pub fn from_mount_dentry(mnt: Arc<super::VfsMount>, dentry: Arc<super::Dentry>) -> Self {
		Self {
			mnt: Some(mnt),
			dentry: Some(dentry),
		}
	}

	/// Get the full path string
	pub fn to_string(&self) -> String {
		if let Some(ref dentry) = self.dentry {
			dentry.get_path()
		} else {
			String::from("/")
		}
	}

	/// Check if path is absolute
	pub fn is_absolute(&self) -> bool {
		self.to_string().starts_with('/')
	}

	/// Get parent path
	pub fn parent(&self) -> Option<Path> {
		if let Some(ref dentry) = self.dentry {
			if let Some(ref parent) = dentry.d_parent {
				Some(Path {
					mnt: self.mnt.clone(),
					dentry: Some(parent.clone()),
				})
			} else {
				None
			}
		} else {
			None
		}
	}

	/// Get filename component
	pub fn filename(&self) -> Option<String> {
		if let Some(ref dentry) = self.dentry {
			Some(dentry.d_name.clone())
		} else {
			None
		}
	}

	/// Join with another path component
	pub fn join(&self, component: &str) -> Result<Path> {
		if let Some(ref dentry) = self.dentry {
			if let Some(child) = dentry.find_child(component) {
				Ok(Path {
					mnt: self.mnt.clone(),
					dentry: Some(child),
				})
			} else {
				Err(Error::ENOENT)
			}
		} else {
			Err(Error::ENOENT)
		}
	}
}

/// Path lookup flags
pub const LOOKUP_FOLLOW: u32 = 0x0001;
pub const LOOKUP_DIRECTORY: u32 = 0x0002;
pub const LOOKUP_AUTOMOUNT: u32 = 0x0004;
pub const LOOKUP_EMPTY: u32 = 0x0008;
pub const LOOKUP_OPEN: u32 = 0x0010;
pub const LOOKUP_CREATE: u32 = 0x0020;
pub const LOOKUP_EXCL: u32 = 0x0040;
pub const LOOKUP_RENAME_TARGET: u32 = 0x0080;

/// Name data structure for path resolution
pub struct NameData {
	/// Path components
	pub path: String,
	/// Current position in path
	pub pos: usize,
	/// Lookup flags
	pub flags: u32,
	/// Root directory
	pub root: Option<Path>,
	/// Current working directory
	pub pwd: Option<Path>,
	/// Result path
	pub result: Option<Path>,
	/// Intent (for create/open operations)
	pub intent: Option<Intent>,
}

/// Intent for path operations
#[derive(Debug, Clone)]
pub enum Intent {
	Open { flags: u32, mode: u32 },
	Create { mode: u32 },
	Lookup,
}

impl NameData {
	/// Create new name data for path resolution
	pub fn new(path: String, flags: u32) -> Self {
		Self {
			path,
			pos: 0,
			flags,
			root: None,
			pwd: None,
			result: None,
			intent: None,
		}
	}

	/// Set root directory
	pub fn with_root(mut self, root: Path) -> Self {
		self.root = Some(root);
		self
	}

	/// Set current working directory
	pub fn with_pwd(mut self, pwd: Path) -> Self {
		self.pwd = Some(pwd);
		self
	}

	/// Set intent
	pub fn with_intent(mut self, intent: Intent) -> Self {
		self.intent = Some(intent);
		self
	}

	/// Get next path component
	pub fn next_component(&mut self) -> Option<String> {
		if self.pos >= self.path.len() {
			return None;
		}

		// Skip leading slashes
		while self.pos < self.path.len() && self.path.chars().nth(self.pos) == Some('/') {
			self.pos += 1;
		}

		if self.pos >= self.path.len() {
			return None;
		}

		// Find end of component
		let start = self.pos;
		while self.pos < self.path.len() && self.path.chars().nth(self.pos) != Some('/') {
			self.pos += 1;
		}

		Some(self.path[start..self.pos].to_string())
	}

	/// Check if path is finished
	pub fn is_finished(&self) -> bool {
		self.pos >= self.path.len()
	}
}

/// Resolve a path to a dentry
pub fn path_lookup(pathname: &str, flags: u32) -> Result<Path> {
	let mut nd = NameData::new(String::from(pathname), flags);

	// Set root directory (for now, use a dummy root)
	// TODO: Get actual root from current process

	// Start from root or current directory
	let mut current_path = if pathname.starts_with('/') {
		// Absolute path - start from root
		if let Some(root) = nd.root.clone() {
			root
		} else {
			// Create dummy root path
			Path::new()
		}
	} else {
		// Relative path - start from current directory
		if let Some(pwd) = nd.pwd.clone() {
			pwd
		} else {
			// Create dummy current directory
			Path::new()
		}
	};

	// Resolve each component
	while let Some(component) = nd.next_component() {
		match component.as_str() {
			"." => {
				// Current directory - no change
				continue;
			}
			".." => {
				// Parent directory
				if let Some(parent) = current_path.parent() {
					current_path = parent;
				}
				continue;
			}
			_ => {
				// Regular component
				current_path = current_path.join(&component)?;
			}
		}

		// Check for mount points
		if let Some(mount) = super::mount::path_get_mount(&current_path.to_string()) {
			current_path.mnt = Some(mount);
		}

		// Handle symlinks if LOOKUP_FOLLOW is set
		if (flags & LOOKUP_FOLLOW) != 0 {
			if let Some(ref dentry) = current_path.dentry {
				if let Some(ref inode) = dentry.d_inode {
					if super::mode::s_islnk(
						inode.i_mode.load(
							core::sync::atomic::Ordering::Relaxed,
						),
					) {
						// TODO: Follow symbolic link
						// For now, just continue
					}
				}
			}
		}
	}

	nd.result = Some(current_path.clone());
	Ok(current_path)
}

/// Resolve parent directory and filename
pub fn path_parent_and_name(pathname: &str) -> Result<(Path, String)> {
	let path = Path::new();

	// Split pathname into parent and filename
	if let Some(last_slash) = pathname.rfind('/') {
		let parent_path = &pathname[..last_slash];
		let filename = &pathname[last_slash + 1..];

		if parent_path.is_empty() {
			// Root directory
			Ok((path, String::from(filename)))
		} else {
			let parent = path_lookup(parent_path, 0)?;
			Ok((parent, String::from(filename)))
		}
	} else {
		// No slash - filename in current directory
		Ok((path, String::from(pathname)))
	}
}

/// Normalize a path (remove . and .. components)
pub fn normalize_path(path: &str) -> String {
	let mut components = Vec::new();

	for component in path.split('/') {
		match component {
			"" | "." => {
				// Skip empty and current directory components
				continue;
			}
			".." => {
				// Parent directory - remove last component
				components.pop();
			}
			_ => {
				// Regular component
				components.push(component);
			}
		}
	}

	let result = components.join("/");
	if path.starts_with('/') {
		format!("/{}", result)
	} else {
		result
	}
}

/// Check if a path is safe (no .. escapes)
pub fn is_safe_path(path: &str) -> bool {
	let normalized = normalize_path(path);

	// Check for .. at the beginning
	if normalized.starts_with("..") {
		return false;
	}

	// Check for /../ sequences
	if normalized.contains("/../") {
		return false;
	}

	true
}

/// Join two paths
pub fn join_paths(base: &str, path: &str) -> String {
	if path.starts_with('/') {
		// Absolute path
		String::from(path)
	} else {
		// Relative path
		let base = base.trim_end_matches('/');
		if base.is_empty() {
			format!("/{}", path)
		} else {
			format!("{}/{}", base, path)
		}
	}
}

/// Get the directory part of a path
pub fn dirname(path: &str) -> &str {
	if let Some(last_slash) = path.rfind('/') {
		if last_slash == 0 {
			"/"
		} else {
			&path[..last_slash]
		}
	} else {
		"."
	}
}

/// Get the filename part of a path
pub fn basename(path: &str) -> &str {
	if let Some(last_slash) = path.rfind('/') {
		&path[last_slash + 1..]
	} else {
		path
	}
}

/// Get file extension
pub fn extension(path: &str) -> Option<&str> {
	let filename = basename(path);
	if let Some(last_dot) = filename.rfind('.') {
		if last_dot > 0 {
			Some(&filename[last_dot + 1..])
		} else {
			None
		}
	} else {
		None
	}
}

/// Check if path is absolute
pub fn is_absolute(path: &str) -> bool {
	path.starts_with('/')
}

/// Check if path is relative
pub fn is_relative(path: &str) -> bool {
	!is_absolute(path)
}
