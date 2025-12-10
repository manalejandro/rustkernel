// SPDX-License-Identifier: GPL-2.0

//! Error handling types and utilities

use core::fmt;

/// Kernel error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
	/// Out of memory
	OutOfMemory,
	/// Invalid argument
	InvalidArgument,
	/// Permission denied
	PermissionDenied,
	/// Resource busy
	Busy,
	/// Resource busy (alias for IPC)
	ResourceBusy,
	/// Resource not found
	NotFound,
	/// Resource already exists
	AlreadyExists,
	/// Operation not supported
	NotSupported,
	/// I/O error
	Io,
	/// Generic I/O error (EIO)
	EIO,
	/// Interrupted operation
	Interrupted,
	/// Resource temporarily unavailable
	WouldBlock,
	/// Device error
	Device,
	/// Generic error
	Generic,
	/// Invalid operation
	InvalidOperation,
	/// Timeout
	Timeout,
	/// Not initialized
	NotInitialized, // New error variant
	/// Network unreachable
	NetworkUnreachable,
	/// Network is down
	NetworkDown,
	/// Device not found
	DeviceNotFound,
	/// Out of memory (ENOMEM)
	ENOMEM,
	/// Host unreachable (EHOSTUNREACH)
	EHOSTUNREACH,

	// Linux-compatible errno values
	/// Operation not permitted (EPERM)
	EPERM,
	/// No such file or directory (ENOENT)
	ENOENT,
	/// Bad file descriptor (EBADF)
	EBADF,
	/// No such device (ENODEV)
	ENODEV,
	/// Invalid argument (EINVAL)
	EINVAL,
	/// No space left on device (ENOSPC)
	ENOSPC,
	/// Inappropriate ioctl for device (ENOTTY)
	ENOTTY,
	/// Illegal seek (ESPIPE)
	ESPIPE,
	/// No data available (ENODATA)
	ENODATA,
	/// Function not implemented (ENOSYS)
	ENOSYS,
	/// Not a directory (ENOTDIR)
	ENOTDIR,
	/// Is a directory (EISDIR)
	EISDIR,
	/// File exists (EEXIST)
	EEXIST,
	/// Directory not empty (ENOTEMPTY)
	ENOTEMPTY,
	/// No child process (ECHILD)
	ECHILD,
	/// No such process (ESRCH)
	ESRCH,
}

impl Error {
	/// Convert error to errno value
	pub fn to_errno(self) -> i32 {
		match self {
			Error::OutOfMemory => -12,     // ENOMEM
			Error::InvalidArgument => -22, // EINVAL
			Error::PermissionDenied => -1, // EPERM
			Error::Busy => -16,            // EBUSY
			Error::ResourceBusy => -16,    // EBUSY (alias)
			Error::NotFound => -2,         // ENOENT
			Error::AlreadyExists => -17,   // EEXIST
			Error::NotSupported => -38,    // ENOSYS
			Error::Io => -5,               // EIO
			Error::Interrupted => -4,      // EINTR
			Error::WouldBlock => -11,      // EAGAIN
			Error::Device => -19,          // ENODEV
			Error::Generic => -1,          // EPERM
			Error::InvalidOperation => -1, // EPERM
			Error::Timeout => -110,        // ETIMEDOUT
			Error::NotInitialized => -6,   // ENXIO

			// Linux errno mappings
			Error::EPERM => -1,                // EPERM
			Error::ENOENT => -2,               // ENOENT
			Error::EBADF => -9,                // EBADF
			Error::ENODEV => -19,              // ENODEV
			Error::EINVAL => -22,              // EINVAL
			Error::ENOSPC => -28,              // ENOSPC
			Error::ENOTTY => -25,              // ENOTTY
			Error::ESPIPE => -29,              // ESPIPE
			Error::ENODATA => -61,             // ENODATA
			Error::ENOSYS => -38,              // ENOSYS
			Error::ENOTDIR => -20,             // ENOTDIR
			Error::EISDIR => -21,              // EISDIR
			Error::EEXIST => -17,              // EEXIST
			Error::ENOTEMPTY => -39,           // ENOTEMPTY
			Error::ECHILD => -10,              // ECHILD
			Error::ESRCH => -3,                // ESRCH
			Error::NetworkUnreachable => -101, // ENETUNREACH
			Error::NetworkDown => -100,        // ENETDOWN
			Error::DeviceNotFound => -19,      // ENODEV
			Error::ENOMEM => -12,              // ENOMEM
			Error::EHOSTUNREACH => -113,       // EHOSTUNREACH
			Error::EIO => -5,                  // EIO
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::OutOfMemory => write!(f, "Out of memory"),
			Error::InvalidArgument => write!(f, "Invalid argument"),
			Error::PermissionDenied => write!(f, "Permission denied"),
			Error::Busy => write!(f, "Resource busy"),
			Error::ResourceBusy => write!(f, "Resource busy"),
			Error::NotFound => write!(f, "Resource not found"),
			Error::AlreadyExists => write!(f, "Resource already exists"),
			Error::NotSupported => write!(f, "Operation not supported"),
			Error::Io => write!(f, "I/O error"),
			Error::Interrupted => write!(f, "Interrupted operation"),
			Error::WouldBlock => write!(f, "Resource temporarily unavailable"),
			Error::Device => write!(f, "Device error"),
			Error::Generic => write!(f, "Generic error"),
			Error::InvalidOperation => write!(f, "Invalid operation"),
			Error::Timeout => write!(f, "Operation timed out"),
			Error::NotInitialized => write!(f, "Not initialized"),
			Error::NetworkUnreachable => write!(f, "Network unreachable"),
			Error::NetworkDown => write!(f, "Network is down"),
			Error::DeviceNotFound => write!(f, "Device not found"),
			Error::ENOMEM => write!(f, "Out of memory"),
			Error::EHOSTUNREACH => write!(f, "Host unreachable"),

			// Linux errno variants
			Error::EPERM => write!(f, "Operation not permitted"),
			Error::ENOENT => write!(f, "No such file or directory"),
			Error::EBADF => write!(f, "Bad file descriptor"),
			Error::ENODEV => write!(f, "No such device"),
			Error::EINVAL => write!(f, "Invalid argument"),
			Error::ENOSPC => write!(f, "No space left on device"),
			Error::ENOTTY => write!(f, "Inappropriate ioctl for device"),
			Error::ESPIPE => write!(f, "Illegal seek"),
			Error::ENODATA => write!(f, "No data available"),
			Error::ENOSYS => write!(f, "Function not implemented"),
			Error::ENOTDIR => write!(f, "Not a directory"),
			Error::EISDIR => write!(f, "Is a directory"),
			Error::EEXIST => write!(f, "File exists"),
			Error::ENOTEMPTY => write!(f, "Directory not empty"),
			Error::ECHILD => write!(f, "No child processes"),
			Error::ESRCH => write!(f, "No such process"),
			Error::EIO => write!(f, "Input/output error"),
		}
	}
}

/// Kernel result type
pub type Result<T> = core::result::Result<T, Error>;

/// Convert from various error types
impl From<()> for Error {
	fn from(_: ()) -> Self {
		Error::Generic
	}
}

impl From<core::alloc::AllocError> for Error {
	fn from(_: core::alloc::AllocError) -> Self {
		Error::OutOfMemory
	}
}
