// SPDX-License-Identifier: GPL-2.0

//! Kernel modules library
//!
//! This crate contains various kernel modules for the Rust kernel.

#![no_std]

extern crate alloc;

pub mod hello;
pub mod test;

pub use hello::*;
pub use test::*;
