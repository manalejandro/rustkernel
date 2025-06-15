// SPDX-License-Identifier: GPL-2.0

//! Heap allocator implementation

use crate::memory::ALLOCATOR;
use crate::types::{VirtAddr, PAGE_SIZE};
use crate::error::Result;

/// Heap start address (will be set during initialization)
static mut HEAP_START: usize = 0;
static mut HEAP_SIZE: usize = 0;

/// Initialize the heap allocator
pub fn init() -> Result<()> {
    // TODO: Get heap region from memory map
    // For now, use a fixed region
    let heap_start = 0x_4444_4444_0000;
    let heap_size = 100 * 1024; // 100 KB
    
    unsafe {
        HEAP_START = heap_start;
        HEAP_SIZE = heap_size;
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
    
    Ok(())
}

/// Get heap statistics
pub fn heap_stats() -> (usize, usize) {
    unsafe { (HEAP_START, HEAP_SIZE) }
}
