// SPDX-License-Identifier: GPL-2.0

//! Advanced file system operations and utilities

use crate::error::{Error, Result};
use crate::fs::file::File;
use crate::fs::inode::Inode;
use crate::fs::dentry::Dentry;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use alloc::sync::Arc;

/// File system statistics
#[derive(Debug, Clone)]
pub struct FsStats {
    pub total_inodes: u64,
    pub free_inodes: u64,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub block_size: u32,
    pub max_filename_len: u32,
    pub filesystem_type: String,
}

/// Directory entry information
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub inode_number: u64,
    pub file_type: u8,
}

/// File system operations
pub struct AdvancedFsOps;

impl AdvancedFsOps {
    /// Create a new file with specified permissions
    pub fn create_file(path: &str, mode: u32) -> Result<Arc<File>> {
        crate::info!("Creating file: {} with mode: {:o}", path, mode);
        
        // Parse path and get parent directory
        let (parent_path, filename) = split_path(path)?;
        
        // Get parent directory inode
        let parent_inode = crate::fs::path::path_lookup(parent_path)?;
        
        // Create new inode for file
        let new_inode = crate::fs::inode::alloc_inode()?;
        new_inode.set_mode(mode | crate::fs::mode::S_IFREG);
        new_inode.set_size(0);
        
        // Create dentry
        let dentry = Arc::new(Dentry::new(filename.to_string(), new_inode.clone()));
        
        // Add to parent directory
        parent_inode.add_child(dentry)?;
        
        // Create file handle
        let file = Arc::new(File::new(new_inode, crate::fs::file::O_RDWR));
        
        Ok(file)
    }
    
    /// Create a new directory
    pub fn create_directory(path: &str, mode: u32) -> Result<()> {
        crate::info!("Creating directory: {} with mode: {:o}", path, mode);
        
        let (parent_path, dirname) = split_path(path)?;
        let parent_inode = crate::fs::path::path_lookup(parent_path)?;
        
        // Create new inode for directory
        let new_inode = crate::fs::inode::alloc_inode()?;
        new_inode.set_mode(mode | crate::fs::mode::S_IFDIR);
        new_inode.set_size(0);
        
        // Create dentry
        let dentry = Arc::new(Dentry::new(dirname.to_string(), new_inode.clone()));
        
        // Add to parent directory
        parent_inode.add_child(dentry)?;
        
        Ok(())
    }
    
    /// Remove a file or directory
    pub fn remove_path(path: &str) -> Result<()> {
        crate::info!("Removing path: {}", path);
        
        let (parent_path, filename) = split_path(path)?;
        let parent_inode = crate::fs::path::path_lookup(parent_path)?;
        
        // Find and remove the child
        parent_inode.remove_child(filename)?;
        
        Ok(())
    }
    
    /// List directory contents
    pub fn list_directory(path: &str) -> Result<Vec<DirEntry>> {
        let inode = crate::fs::path::path_lookup(path)?;
        
        if !inode.is_dir() {
            return Err(Error::ENOTDIR);
        }
        
        let mut entries = Vec::new();
        
        // Add . and .. entries
        entries.push(DirEntry {
            name: ".".to_string(),
            inode_number: inode.get_ino(),
            file_type: crate::fs::mode::DT_DIR,
        });
        
        if let Some(parent) = inode.get_parent() {
            entries.push(DirEntry {
                name: "..".to_string(),
                inode_number: parent.get_ino(),
                file_type: crate::fs::mode::DT_DIR,
            });
        }
        
        // Add actual directory entries
        for child in inode.get_children() {
            let file_type = if child.inode.is_dir() {
                crate::fs::mode::DT_DIR
            } else {
                crate::fs::mode::DT_REG
            };
            
            entries.push(DirEntry {
                name: child.name.clone(),
                inode_number: child.inode.get_ino(),
                file_type,
            });
        }
        
        Ok(entries)
    }
    
    /// Get file/directory statistics
    pub fn get_stats(path: &str) -> Result<crate::fs::inode::Stat> {
        let inode = crate::fs::path::path_lookup(path)?;
        Ok(inode.get_stat())
    }
    
    /// Get file system statistics
    pub fn get_fs_stats(path: &str) -> Result<FsStats> {
        let _inode = crate::fs::path::path_lookup(path)?;
        
        // Return simplified stats
        Ok(FsStats {
            total_inodes: 1000000,
            free_inodes: 999000,
            total_blocks: 1000000,
            free_blocks: 900000,
            block_size: 4096,
            max_filename_len: 255,
            filesystem_type: "RustFS".to_string(),
        })
    }
    
    /// Copy file
    pub fn copy_file(src: &str, dst: &str) -> Result<()> {
        crate::info!("Copying file from {} to {}", src, dst);
        
        // Open source file
        let src_inode = crate::fs::path::path_lookup(src)?;
        if src_inode.is_dir() {
            return Err(Error::EISDIR);
        }
        
        // Read source file data
        let mut buffer = vec![0u8; src_inode.get_size() as usize];
        src_inode.read_at(0, &mut buffer)?;
        
        // Create destination file
        let dst_file = Self::create_file(dst, 0o644)?;
        
        // Write data to destination
        dst_file.write(&buffer)?;
        
        Ok(())
    }
    
    /// Move/rename file
    pub fn move_file(src: &str, dst: &str) -> Result<()> {
        crate::info!("Moving file from {} to {}", src, dst);
        
        // Copy file
        Self::copy_file(src, dst)?;
        
        // Remove source
        Self::remove_path(src)?;
        
        Ok(())
    }
    
    /// Create symbolic link
    pub fn create_symlink(target: &str, link: &str) -> Result<()> {
        crate::info!("Creating symlink: {} -> {}", link, target);
        
        let (parent_path, linkname) = split_path(link)?;
        let parent_inode = crate::fs::path::path_lookup(parent_path)?;
        
        // Create new inode for symlink
        let new_inode = crate::fs::inode::alloc_inode()?;
        new_inode.set_mode(0o777 | crate::fs::mode::S_IFLNK);
        new_inode.set_size(target.len() as u64);
        
        // Store target path in inode data
        new_inode.write_at(0, target.as_bytes())?;
        
        // Create dentry
        let dentry = Arc::new(Dentry::new(linkname.to_string(), new_inode.clone()));
        
        // Add to parent directory
        parent_inode.add_child(dentry)?;
        
        Ok(())
    }
    
    /// Create hard link
    pub fn create_hardlink(target: &str, link: &str) -> Result<()> {
        crate::info!("Creating hardlink: {} -> {}", link, target);
        
        let target_inode = crate::fs::path::path_lookup(target)?;
        let (parent_path, linkname) = split_path(link)?;
        let parent_inode = crate::fs::path::path_lookup(parent_path)?;
        
        // Create dentry pointing to existing inode
        let dentry = Arc::new(Dentry::new(linkname.to_string(), target_inode.clone()));
        
        // Add to parent directory
        parent_inode.add_child(dentry)?;
        
        // Increment link count
        target_inode.inc_nlink();
        
        Ok(())
    }
}

/// Split path into parent and filename
fn split_path(path: &str) -> Result<(&str, &str)> {
    if path == "/" {
        return Err(Error::EINVAL);
    }
    
    let path = path.trim_end_matches('/');
    if let Some(pos) = path.rfind('/') {
        let parent = if pos == 0 { "/" } else { &path[..pos] };
        let filename = &path[pos + 1..];
        Ok((parent, filename))
    } else {
        Ok(("/", path))
    }
}

/// File system utility functions
pub mod utils {
    use super::*;
    
    /// Check if path exists
    pub fn path_exists(path: &str) -> bool {
        crate::fs::path::path_lookup(path).is_ok()
    }
    
    /// Check if path is a directory
    pub fn is_directory(path: &str) -> bool {
        if let Ok(inode) = crate::fs::path::path_lookup(path) {
            inode.is_dir()
        } else {
            false
        }
    }
    
    /// Check if path is a regular file
    pub fn is_regular_file(path: &str) -> bool {
        if let Ok(inode) = crate::fs::path::path_lookup(path) {
            inode.is_file()
        } else {
            false
        }
    }
    
    /// Get file size
    pub fn get_file_size(path: &str) -> Result<u64> {
        let inode = crate::fs::path::path_lookup(path)?;
        Ok(inode.get_size())
    }
    
    /// Get file permissions
    pub fn get_file_mode(path: &str) -> Result<u32> {
        let inode = crate::fs::path::path_lookup(path)?;
        Ok(inode.get_mode())
    }
    
    /// Set file permissions
    pub fn set_file_mode(path: &str, mode: u32) -> Result<()> {
        let inode = crate::fs::path::path_lookup(path)?;
        inode.set_mode(mode);
        Ok(())
    }
}

/// Initialize advanced file system operations
pub fn init() -> Result<()> {
    crate::info!("Advanced file system operations initialized");
    Ok(())
}
