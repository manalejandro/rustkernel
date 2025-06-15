// SPDX-License-Identifier: GPL-2.0

//! Virtual memory allocation

use crate::error::Result;
use crate::types::VirtAddr;

/// Allocate virtual memory
pub fn vmalloc(size: usize) -> Result<VirtAddr> {
    // TODO: implement proper vmalloc  
    Ok(VirtAddr::new(0))
}

/// Free virtual memory
pub fn vfree(addr: VirtAddr) {
    // TODO: implement proper vfree
}
