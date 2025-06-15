// SPDX-License-Identifier: GPL-2.0

//! Kernel memory allocation (kmalloc)

use crate::error::Result;

/// Allocate kernel memory
pub fn kmalloc(size: usize) -> Result<*mut u8> {
    // TODO: implement proper kmalloc
    Ok(core::ptr::null_mut())
}

/// Free kernel memory
pub fn kfree(ptr: *mut u8) {
    // TODO: implement proper kfree
}
