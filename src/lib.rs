// SPDX-License-Identifier: GPL-2.0

//! Rust Kernel Library
//!
//! A modern kernel implementation in Rust, inspired by the Linux kernel
//! and utilizing the Rust for Linux infrastructure.

#![no_std]

extern crate kernel;

// Re-export the kernel crate
pub use kernel::*;

#[cfg(test)]
mod tests {
    #[test]
    fn basic_test() {
        assert_eq!(2 + 2, 4);
    }
}
