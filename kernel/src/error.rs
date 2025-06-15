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
    /// Resource not found
    NotFound,
    /// Operation not supported
    NotSupported,
    /// I/O error
    Io,
    /// Interrupted operation
    Interrupted,
    /// Resource temporarily unavailable
    WouldBlock,
    /// Device error
    Device,
    /// Generic error
    Generic,
}

impl Error {
    /// Convert error to errno value
    pub fn to_errno(self) -> i32 {
        match self {
            Error::OutOfMemory => -12,        // ENOMEM
            Error::InvalidArgument => -22,    // EINVAL
            Error::PermissionDenied => -1,    // EPERM
            Error::Busy => -16,               // EBUSY
            Error::NotFound => -2,            // ENOENT
            Error::NotSupported => -38,       // ENOSYS
            Error::Io => -5,                  // EIO
            Error::Interrupted => -4,         // EINTR
            Error::WouldBlock => -11,         // EAGAIN
            Error::Device => -19,             // ENODEV
            Error::Generic => -1,             // EPERM
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
            Error::NotFound => write!(f, "Resource not found"),
            Error::NotSupported => write!(f, "Operation not supported"),
            Error::Io => write!(f, "I/O error"),
            Error::Interrupted => write!(f, "Interrupted operation"),
            Error::WouldBlock => write!(f, "Resource temporarily unavailable"),
            Error::Device => write!(f, "Device error"),
            Error::Generic => write!(f, "Generic error"),
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
