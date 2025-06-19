// SPDX-License-Identifier: GPL-2.0

//! Kernel module support

use crate::error::Result;

/// Module metadata
pub struct ThisModule {
	pub name: &'static str,
	pub author: &'static str,
	pub description: &'static str,
	pub license: &'static str,
}

/// Trait for kernel modules
pub trait Module: Sized {
	/// Initialize the module
	fn init(module: &'static ThisModule) -> Result<Self>;

	/// Clean up the module
	fn exit(module: &'static ThisModule) {
		// Default implementation does nothing
	}
}
