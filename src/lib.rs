// SPDX-License-Identifier: GPL-2.0

//! Rust Kernel
//!
//! A modern kernel implementation in Rust, inspired by the Linux kernel
//! and utilizing the Rust for Linux infrastructure.

#![no_std]
#![no_main]

// Re-export the kernel crate
pub use kernel;

/// Main kernel entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel::kernel_main()
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_test() {
        assert_eq!(2 + 2, 4);
    }
}
