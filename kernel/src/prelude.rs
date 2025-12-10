// SPDX-License-Identifier: GPL-2.0

//! Kernel prelude - commonly used types and traits

// Re-export macros
pub use alloc::vec;
// Re-export common alloc types
pub use alloc::{
	boxed::Box,
	collections::{BTreeMap, BTreeSet},
	format,
	string::{String, ToString},
	vec::Vec,
};
// Re-export core types
pub use core::{
	fmt, mem,
	option::Option::{self, None, Some},
	ptr,
	result::Result as CoreResult,
	slice, str,
};

pub use crate::device::Device;
pub use crate::driver::{BlockDriverOps, CharDriverOps, Driver};
pub use crate::error::{Error, Result};
pub use crate::memory::{PageTable, PhysAddr, UserPtr, UserSlicePtr, VirtAddr};
pub use crate::process::{Process, Thread};
pub use crate::sync::{Mutex, RwLock, Spinlock};
pub use crate::task::Task;
pub use crate::types::*;

/// Print macros for kernel logging
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::console::_kprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("[KERNEL] {}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug")]
        $crate::kprintln!("[DEBUG] {}", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ($crate::kprintln!("[INFO] {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ($crate::kprintln!("[WARN] {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ($crate::kprintln!("[ERROR] {}", format_args!($($arg)*)));
}

/// Module definition macro
#[macro_export]
macro_rules! module {
	(
		type:
		$type:ty,name:
		$name:expr,author:
		$author:expr,description:
		$description:expr,license:
		$license:expr $(,)?
	) => {
		static __THIS_MODULE: $crate::module::ThisModule = $crate::module::ThisModule {
			name: $name,
			author: $author,
			description: $description,
			license: $license,
		};

		#[no_mangle]
		pub extern "C" fn init_module() -> core::ffi::c_int {
			match <$type as $crate::module::Module>::init(&__THIS_MODULE) {
				Ok(_) => 0,
				Err(e) => e.to_errno(),
			}
		}

		#[no_mangle]
		pub extern "C" fn cleanup_module() {
			<$type as $crate::module::Module>::exit(&__THIS_MODULE)
		}
	};
}
