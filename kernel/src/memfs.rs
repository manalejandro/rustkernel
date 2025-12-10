// SPDX-License-Identifier: GPL-2.0

//! Simple in-memory file system

use alloc::{
	boxed::Box,
	collections::BTreeMap,
	string::{String, ToString},
	vec,
	vec::Vec,
};

use crate::error::Result;
use crate::sync::Spinlock;
use crate::{error, info, warn};

/// File type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
	RegularFile,
	Directory,
	SymbolicLink,
	CharDevice,
	BlockDevice,
}

/// File permissions (simplified)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(pub u32);

impl FileMode {
	pub const READ: u32 = 0o444;
	pub const WRITE: u32 = 0o222;
	pub const EXECUTE: u32 = 0o111;
	pub const ALL: u32 = 0o777;

	pub fn new(mode: u32) -> Self {
		Self(mode)
	}

	pub fn can_read(&self) -> bool {
		self.0 & Self::READ != 0
	}

	pub fn can_write(&self) -> bool {
		self.0 & Self::WRITE != 0
	}

	pub fn can_execute(&self) -> bool {
		self.0 & Self::EXECUTE != 0
	}
}

/// In-memory file node
#[derive(Debug)]
pub struct MemFile {
	pub name: String,
	pub file_type: FileType,
	pub mode: FileMode,
	pub size: usize,
	pub data: Vec<u8>,
	pub children: BTreeMap<String, Box<MemFile>>,
	pub parent: Option<String>,
}

impl MemFile {
	pub fn new_file(name: String, mode: FileMode) -> Self {
		Self {
			name,
			file_type: FileType::RegularFile,
			mode,
			size: 0,
			data: Vec::new(),
			children: BTreeMap::new(),
			parent: None,
		}
	}

	pub fn new_dir(name: String, mode: FileMode) -> Self {
		Self {
			name,
			file_type: FileType::Directory,
			mode,
			size: 0,
			data: Vec::new(),
			children: BTreeMap::new(),
			parent: None,
		}
	}

	pub fn is_dir(&self) -> bool {
		self.file_type == FileType::Directory
	}

	pub fn is_file(&self) -> bool {
		self.file_type == FileType::RegularFile
	}

	/// Write data to file
	pub fn write(&mut self, data: &[u8]) -> Result<usize> {
		if !self.mode.can_write() {
			return Err(crate::error::Error::PermissionDenied);
		}

		if !self.is_file() {
			return Err(crate::error::Error::InvalidOperation);
		}

		self.data.extend_from_slice(data);
		self.size = self.data.len();
		Ok(data.len())
	}

	/// Read data from file
	pub fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<usize> {
		if !self.mode.can_read() {
			return Err(crate::error::Error::PermissionDenied);
		}

		if !self.is_file() {
			return Err(crate::error::Error::InvalidOperation);
		}

		if offset >= self.data.len() {
			return Ok(0);
		}

		let available = self.data.len() - offset;
		let to_read = buffer.len().min(available);

		buffer[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
		Ok(to_read)
	}

	/// Add child to directory
	pub fn add_child(&mut self, child: MemFile) -> Result<()> {
		if !self.is_dir() {
			return Err(crate::error::Error::InvalidOperation);
		}

		let name = child.name.clone();
		self.children.insert(name, Box::new(child));
		Ok(())
	}

	/// Remove child from directory
	pub fn remove_child(&mut self, name: &str) -> Result<()> {
		if !self.is_dir() {
			return Err(crate::error::Error::InvalidOperation);
		}

		if self.children.remove(name).is_some() {
			Ok(())
		} else {
			Err(crate::error::Error::NotFound)
		}
	}

	/// Get child by name
	pub fn get_child(&self, name: &str) -> Option<&MemFile> {
		self.children.get(name).map(|f| f.as_ref())
	}

	/// Get child by name (mutable)
	pub fn get_child_mut(&mut self, name: &str) -> Option<&mut MemFile> {
		self.children.get_mut(name).map(|f| f.as_mut())
	}

	/// List directory contents
	pub fn list_children(&self) -> Vec<&str> {
		if !self.is_dir() {
			return Vec::new();
		}

		self.children.keys().map(|s| s.as_str()).collect()
	}
}

/// Simple in-memory file system
pub struct MemFileSystem {
	root: MemFile,
	current_dir: String,
}

impl MemFileSystem {
	pub fn new() -> Self {
		let root = MemFile::new_dir("/".to_string(), FileMode::new(FileMode::ALL));

		Self {
			root,
			current_dir: "/".to_string(),
		}
	}

	/// Initialize with some default files
	pub fn init_default_files(&mut self) -> Result<()> {
		// Create /proc directory
		let proc_dir = MemFile::new_dir(
			"proc".to_string(),
			FileMode::new(FileMode::READ | FileMode::EXECUTE),
		);
		self.root.add_child(proc_dir)?;

		// Create /tmp directory
		let tmp_dir = MemFile::new_dir("tmp".to_string(), FileMode::new(FileMode::ALL));
		self.root.add_child(tmp_dir)?;

		// Create /dev directory
		let dev_dir = MemFile::new_dir("dev".to_string(), FileMode::new(FileMode::ALL));
		self.root.add_child(dev_dir)?;

		// Create some example files
		let mut readme =
			MemFile::new_file("README.txt".to_string(), FileMode::new(FileMode::READ));
		readme.write(
			b"Welcome to the Rust Kernel!\nThis is a simple in-memory file system.\n",
		)?;
		self.root.add_child(readme)?;

		let mut version =
			MemFile::new_file("version".to_string(), FileMode::new(FileMode::READ));
		version.write(crate::VERSION.as_bytes())?;
		self.root.add_child(version)?;

		info!("Default file system structure created");
		Ok(())
	}

	/// Resolve path to file
	fn resolve_path(&self, path: &str) -> Option<&MemFile> {
		if path == "/" {
			return Some(&self.root);
		}

		let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
		let mut current = &self.root;

		for part in parts {
			if part.is_empty() {
				continue;
			}
			current = current.get_child(part)?;
		}

		Some(current)
	}

	/// Resolve path to file (mutable)
	fn resolve_path_mut(&mut self, path: &str) -> Option<&mut MemFile> {
		if path == "/" {
			return Some(&mut self.root);
		}

		let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
		let mut current = &mut self.root;

		for part in parts {
			if part.is_empty() {
				continue;
			}
			current = current.get_child_mut(part)?;
		}

		Some(current)
	}

	/// Create a file
	pub fn create_file(&mut self, path: &str, mode: FileMode) -> Result<()> {
		let (dir_path, filename) = self.split_path(path);

		if let Some(dir) = self.resolve_path_mut(&dir_path) {
			if !dir.is_dir() {
				return Err(crate::error::Error::InvalidOperation);
			}

			let file = MemFile::new_file(filename, mode);
			dir.add_child(file)?;
			Ok(())
		} else {
			Err(crate::error::Error::NotFound)
		}
	}

	/// Create a directory
	pub fn create_dir(&mut self, path: &str, mode: FileMode) -> Result<()> {
		let (dir_path, dirname) = self.split_path(path);

		if let Some(dir) = self.resolve_path_mut(&dir_path) {
			if !dir.is_dir() {
				return Err(crate::error::Error::InvalidOperation);
			}

			let new_dir = MemFile::new_dir(dirname, mode);
			dir.add_child(new_dir)?;
			Ok(())
		} else {
			Err(crate::error::Error::NotFound)
		}
	}

	/// Write to a file
	pub fn write_file(&mut self, path: &str, data: &[u8]) -> Result<usize> {
		if let Some(file) = self.resolve_path_mut(path) {
			file.write(data)
		} else {
			Err(crate::error::Error::NotFound)
		}
	}

	/// Read from a file
	pub fn read_file(&self, path: &str, offset: usize, buffer: &mut [u8]) -> Result<usize> {
		if let Some(file) = self.resolve_path(path) {
			file.read(offset, buffer)
		} else {
			Err(crate::error::Error::NotFound)
		}
	}

	/// List directory contents
	pub fn list_dir(&self, path: &str) -> Result<Vec<(String, FileType, usize)>> {
		if let Some(dir) = self.resolve_path(path) {
			if !dir.is_dir() {
				return Err(crate::error::Error::InvalidOperation);
			}

			let mut entries = Vec::new();
			for (name, child) in &dir.children {
				entries.push((name.clone(), child.file_type, child.size));
			}
			Ok(entries)
		} else {
			Err(crate::error::Error::NotFound)
		}
	}

	/// Remove a file or directory
	pub fn remove(&mut self, path: &str) -> Result<()> {
		let (dir_path, filename) = self.split_path(path);

		if let Some(dir) = self.resolve_path_mut(&dir_path) {
			dir.remove_child(&filename)
		} else {
			Err(crate::error::Error::NotFound)
		}
	}

	/// Get file info
	pub fn stat(&self, path: &str) -> Result<(FileType, FileMode, usize)> {
		if let Some(file) = self.resolve_path(path) {
			Ok((file.file_type, file.mode, file.size))
		} else {
			Err(crate::error::Error::NotFound)
		}
	}

	/// Split path into directory and filename
	fn split_path(&self, path: &str) -> (String, String) {
		if let Some(pos) = path.rfind('/') {
			let dir = if pos == 0 { "/" } else { &path[..pos] };
			let file = &path[pos + 1..];
			(dir.to_string(), file.to_string())
		} else {
			("/".to_string(), path.to_string())
		}
	}
}

/// Global file system instance
static FILESYSTEM: Spinlock<Option<MemFileSystem>> = Spinlock::new(None);

/// Initialize the in-memory file system
pub fn init_memfs() -> Result<()> {
	info!("Initializing in-memory file system");

	let mut fs = MemFileSystem::new();
	fs.init_default_files()?;

	let mut filesystem = FILESYSTEM.lock();
	*filesystem = Some(fs);

	info!("In-memory file system initialized");
	Ok(())
}

/// File system operations for shell
pub fn fs_list(path: &str) -> Result<Vec<(String, FileType, usize)>> {
	let filesystem = FILESYSTEM.lock();
	if let Some(ref fs) = *filesystem {
		fs.list_dir(path)
	} else {
		Err(crate::error::Error::NotInitialized)
	}
}

pub fn fs_read(path: &str) -> Result<Vec<u8>> {
	let filesystem = FILESYSTEM.lock();
	if let Some(ref fs) = *filesystem {
		let mut buffer = vec![0u8; 4096]; // Read up to 4KB
		let bytes_read = fs.read_file(path, 0, &mut buffer)?;
		buffer.truncate(bytes_read);
		Ok(buffer)
	} else {
		Err(crate::error::Error::NotInitialized)
	}
}

pub fn fs_write(path: &str, data: &[u8]) -> Result<usize> {
	let mut filesystem = FILESYSTEM.lock();
	if let Some(ref mut fs) = *filesystem {
		fs.write_file(path, data)
	} else {
		Err(crate::error::Error::NotInitialized)
	}
}

pub fn fs_create_file(path: &str) -> Result<()> {
	let mut filesystem = FILESYSTEM.lock();
	if let Some(ref mut fs) = *filesystem {
		fs.create_file(path, FileMode::new(FileMode::READ | FileMode::WRITE))
	} else {
		Err(crate::error::Error::NotInitialized)
	}
}

pub fn fs_create_dir(path: &str) -> Result<()> {
	let mut filesystem = FILESYSTEM.lock();
	if let Some(ref mut fs) = *filesystem {
		fs.create_dir(path, FileMode::new(FileMode::ALL))
	} else {
		Err(crate::error::Error::NotInitialized)
	}
}

pub fn fs_remove(path: &str) -> Result<()> {
	let mut filesystem = FILESYSTEM.lock();
	if let Some(ref mut fs) = *filesystem {
		fs.remove(path)
	} else {
		Err(crate::error::Error::NotInitialized)
	}
}

pub fn fs_stat(path: &str) -> Result<(FileType, FileMode, usize)> {
	let filesystem = FILESYSTEM.lock();
	if let Some(ref fs) = *filesystem {
		fs.stat(path)
	} else {
		Err(crate::error::Error::NotInitialized)
	}
}

/// File system statistics for diagnostics
#[derive(Debug, Clone)]
pub struct FileSystemStats {
	pub files_count: usize,
	pub directories_count: usize,
	pub total_size: usize,
}

/// Get file system statistics for diagnostics
pub fn get_filesystem_stats() -> Result<FileSystemStats> {
	let filesystem = FILESYSTEM.lock();
	if let Some(ref fs) = *filesystem {
		let mut files_count = 0;
		let mut directories_count = 0;
		let mut total_size = 0;

		// Count files recursively (simplified implementation)
		fn count_files(
			file: &MemFile,
			files: &mut usize,
			dirs: &mut usize,
			size: &mut usize,
		) {
			if file.is_dir() {
				*dirs += 1;
				for child in file.children.values() {
					count_files(child, files, dirs, size);
				}
			} else {
				*files += 1;
				*size += file.data.len();
			}
		}

		count_files(
			&fs.root,
			&mut files_count,
			&mut directories_count,
			&mut total_size,
		);

		Ok(FileSystemStats {
			files_count,
			directories_count,
			total_size,
		})
	} else {
		Err(crate::error::Error::NotInitialized)
	}
}
