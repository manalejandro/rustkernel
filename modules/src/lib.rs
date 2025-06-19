// SPDX-License-Identifier: GPL-2.0

//! Kernel modules library
//!
//! This crate contains various kernel modules for the Rust kernel.

#![no_std]

extern crate alloc;

// Module infrastructure is ready for dynamic modules
// Modules can be loaded at runtime through the module_loader system
