// SPDX-License-Identifier: GPL-2.0

//! Architecture-specific code

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

// Other architectures can be added here when needed
// #[cfg(target_arch = "aarch64")]
// pub mod aarch64;

// #[cfg(target_arch = "riscv64")]
// pub mod riscv64;
