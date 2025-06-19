// SPDX-License-Identifier: GPL-2.0

//! Network stub for basic functionality

use alloc::string::{String, ToString};

use crate::error::Result;

/// Initialize basic networking
pub fn init() -> Result<()> {
	crate::info!("Network stub initialized");
	Ok(())
}

/// Get network status
pub fn get_network_status() -> String {
	"Network: Basic stub - No interfaces configured".to_string()
}
