// SPDX-License-Identifier: GPL-2.0

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
	println!("cargo:rerun-if-changed=linker.ld");

	let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

	// Copy linker script to OUT_DIR so the linker can find it
	let linker_script = out_dir.join("linker.ld");
	fs::copy("linker.ld", &linker_script).expect("Failed to copy linker script");

	// Tell cargo to pass the linker script to the linker
	println!("cargo:rustc-link-arg=-T{}", linker_script.display());
}
