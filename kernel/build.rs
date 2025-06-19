// SPDX-License-Identifier: GPL-2.0

fn main() {
	// Build assembly files with rustc
	println!("cargo:rerun-if-changed=src/arch/x86_64/boot.s");
	println!("cargo:rerun-if-changed=src/arch/x86_64/exceptions.s");
	println!("cargo:rerun-if-changed=linker.ld");

	// Tell Cargo to link against the linker script
	println!("cargo:rustc-link-arg=-Tlinker.ld");
}
