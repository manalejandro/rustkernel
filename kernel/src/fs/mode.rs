// SPDX-License-Identifier: GPL-2.0

//! File mode utilities - Linux compatible

/// File mode constants (Linux compatible)
pub const S_IFMT: u32 = 0o170000; // File type mask
pub const S_IFSOCK: u32 = 0o140000; // Socket
pub const S_IFLNK: u32 = 0o120000; // Symbolic link
pub const S_IFREG: u32 = 0o100000; // Regular file
pub const S_IFBLK: u32 = 0o060000; // Block device
pub const S_IFDIR: u32 = 0o040000; // Directory
pub const S_IFCHR: u32 = 0o020000; // Character device
pub const S_IFIFO: u32 = 0o010000; // FIFO/pipe

/// Permission bits
pub const S_ISUID: u32 = 0o004000; // Set user ID
pub const S_ISGID: u32 = 0o002000; // Set group ID
pub const S_ISVTX: u32 = 0o001000; // Sticky bit

/// User permissions
pub const S_IRUSR: u32 = 0o000400; // Read by owner
pub const S_IWUSR: u32 = 0o000200; // Write by owner
pub const S_IXUSR: u32 = 0o000100; // Execute by owner

/// Group permissions
pub const S_IRGRP: u32 = 0o000040; // Read by group
pub const S_IWGRP: u32 = 0o000020; // Write by group
pub const S_IXGRP: u32 = 0o000010; // Execute by group

/// Other permissions
pub const S_IROTH: u32 = 0o000004; // Read by others
pub const S_IWOTH: u32 = 0o000002; // Write by others
pub const S_IXOTH: u32 = 0o000001; // Execute by others

/// Linux stat utility functions
pub fn s_isreg(mode: u32) -> bool {
	(mode & S_IFMT) == S_IFREG
}

pub fn s_isdir(mode: u32) -> bool {
	(mode & S_IFMT) == S_IFDIR
}

pub fn s_ischr(mode: u32) -> bool {
	(mode & S_IFMT) == S_IFCHR
}

pub fn s_isblk(mode: u32) -> bool {
	(mode & S_IFMT) == S_IFBLK
}

pub fn s_isfifo(mode: u32) -> bool {
	(mode & S_IFMT) == S_IFIFO
}

pub fn s_islnk(mode: u32) -> bool {
	(mode & S_IFMT) == S_IFLNK
}

pub fn s_issock(mode: u32) -> bool {
	(mode & S_IFMT) == S_IFSOCK
}

/// Check if mode has execute permission for user
pub fn s_ixusr(mode: u32) -> bool {
	(mode & S_IXUSR) != 0
}

/// Check if mode has execute permission for group
pub fn s_ixgrp(mode: u32) -> bool {
	(mode & S_IXGRP) != 0
}

/// Check if mode has execute permission for others
pub fn s_ixoth(mode: u32) -> bool {
	(mode & S_IXOTH) != 0
}

/// Default file mode (0644)
pub const DEFAULT_FILE_MODE: u32 = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;

/// Default directory mode (0755)
pub const DEFAULT_DIR_MODE: u32 =
	S_IRUSR | S_IWUSR | S_IXUSR | S_IRGRP | S_IXGRP | S_IROTH | S_IXOTH;
