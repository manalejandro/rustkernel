# SPDX-License-Identifier: GPL-2.0

# Rust Kernel Makefile

RUST_KERNEL_VERSION := 0.1.0
ARCH ?= x86_64
BUILD_TYPE ?= release

# Enable unstable features on stable compiler
export RUSTC_BOOTSTRAP := 1

# Cargo configuration
CARGO := cargo

ifeq ($(BUILD_TYPE),debug)
    CARGO_FLAGS :=
else
    CARGO_FLAGS := --release
endif

# Kernel build command
KERNEL_BUILD_CMD := cd kernel && $(CARGO) build $(CARGO_FLAGS) \
	--target x86_64-unknown-none \
	-Z build-std=core,alloc \
	-Z build-std-features=compiler-builtins-mem

# Binary locations
KERNEL_BIN := kernel/target/x86_64-unknown-none/$(BUILD_TYPE)/rust-kernel
ISO_BOOT := iso/boot/rust-kernel

# Default target
all: iso

# Build the kernel binary
kernel:
	@echo "Building Rust kernel ($(ARCH), $(BUILD_TYPE))"
	@$(KERNEL_BUILD_CMD)
	@echo "Kernel binary: $(KERNEL_BIN)"

# Create bootable ISO
iso: kernel
	@echo "Creating bootable ISO..."
	@mkdir -p iso/boot/grub
	@cp $(KERNEL_BIN) $(ISO_BOOT)
	@grub-mkrescue -o rust-kernel.iso iso 2>/dev/null && echo "ISO created: rust-kernel.iso"

# Run in QEMU
run: iso
	@echo "Starting kernel in QEMU (Ctrl+C to exit)..."
	@qemu-system-x86_64 -m 512M -cdrom rust-kernel.iso -serial stdio -no-reboot

# Quick test run
test-run: iso
	@echo "Testing kernel (10s timeout)..."
	@timeout 10s qemu-system-x86_64 -m 512M -cdrom rust-kernel.iso -serial stdio -no-reboot 2>&1 || true

# Run with debug output
debug: iso
	@qemu-system-x86_64 -m 512M -cdrom rust-kernel.iso -serial stdio -no-reboot -d int,cpu_reset

# Clean build artifacts
clean:
	@cd kernel && $(CARGO) clean
	@rm -f rust-kernel.iso
	@echo "Clean complete"

# Format code
fmt:
	@cd kernel && $(CARGO) fmt

# Check formatting
fmt-check:
	@cd kernel && $(CARGO) fmt --check

# Generate documentation
doc:
	@cd kernel && $(CARGO) doc --no-deps

.PHONY: all kernel iso run test-run debug clean fmt fmt-check doc
