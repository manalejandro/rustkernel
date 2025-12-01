// SPDX-License-Identifier: GPL-2.0

//! Rust Kernel Binary
//!
//! Main binary entry point for the Rust kernel

#![no_std]
#![no_main]

extern crate kernel;

use core::arch::global_asm;

// Include boot assembly
#[cfg(target_arch = "x86_64")]
global_asm!(
	include_str!("../kernel/src/arch/x86_64/boot.s"),
	options(att_syntax)
);
