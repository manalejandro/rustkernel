# SPDX-License-Identifier: GPL-2.0

# Rust Kernel Makefile
# Based on Linux kernel Rust infrastructure

RUST_KERNEL_VERSION := 0.1.0

# Build configuration
ARCH ?= x86_64
BUILD_TYPE ?= release

# Enable unstable features on stable compiler
export RUSTC_BOOTSTRAP := 1

# Cargo configuration
CARGO := cargo
CARGO_FLAGS := --target-dir target

ifeq ($(BUILD_TYPE),debug)
    CARGO_FLAGS += 
else
    CARGO_FLAGS += --release
endif

# Kernel build command with proper flags
KERNEL_BUILD_CMD := cd kernel && $(CARGO) build $(CARGO_FLAGS) \
	--target x86_64-unknown-none \
	-Z build-std=core,alloc \
	-Z build-std-features=compiler-builtins-mem

# Kernel modules
RUST_MODULES := $(shell find modules -name "*.rs" -type f)
DRIVERS := $(shell find drivers -name "*.rs" -type f)

# Binary locations
KERNEL_BIN := kernel/target/x86_64-unknown-none/$(BUILD_TYPE)/rust-kernel
ISO_BOOT := iso/boot/rust-kernel

# Default target
all: kernel iso

# Build the core kernel
kernel:
	@echo "Building Rust kernel ($(ARCH), $(BUILD_TYPE))"
	$(KERNEL_BUILD_CMD)
	@echo "Kernel binary: $(KERNEL_BIN)"

# Create bootable ISO
iso: kernel
	@echo "Creating bootable ISO..."
	@mkdir -p iso/boot/grub
	@cp $(KERNEL_BIN) $(ISO_BOOT)
	@if command -v grub-mkrescue >/dev/null 2>&1; then \
		grub-mkrescue -o rust-kernel.iso iso && \
		echo "ISO created: rust-kernel.iso"; \
	else \
		echo "Warning: grub-mkrescue not found. Install grub-common and xorriso."; \
	fi

# Build kernel modules
modules: $(RUST_MODULES)
	@echo "Building kernel modules"
	cd modules && $(CARGO) build $(CARGO_FLAGS)

# Build drivers
drivers: $(DRIVERS)
	@echo "Building drivers"
	cd drivers && $(CARGO) build $(CARGO_FLAGS)

# Run in QEMU
run: iso
	@echo "Starting kernel in QEMU..."
	@echo "Press Ctrl+C to exit."
	qemu-system-x86_64 -m 512M -cdrom rust-kernel.iso -serial stdio -no-reboot

# Quick test run with timeout
test-run: iso
	@echo "Testing kernel in QEMU (10 second timeout)..."
	timeout 10s qemu-system-x86_64 -m 512M -cdrom rust-kernel.iso -serial stdio -no-reboot || true

# Clean build artifacts
clean:
	$(CARGO) clean
	cd kernel && $(CARGO) clean
	rm -rf target/
	rm -f rust-kernel.iso

# Run tests
test:
	$(CARGO) test $(CARGO_FLAGS)

# Check formatting
fmt-check:
	$(CARGO) fmt --check

# Format code
fmt:
	$(CARGO) fmt

# Run clippy
clippy:
	$(CARGO) clippy $(CARGO_FLAGS) -- -D warnings

# Generate documentation
doc:
	$(CARGO) doc $(CARGO_FLAGS) --no-deps

# Test the kernel
test-kernel: kernel
	@echo "Testing kernel functionality"
	cd kernel && $(CARGO) test $(CARGO_FLAGS)

# Install (placeholder)
install:
	@echo "Install target not implemented yet"

.PHONY: all kernel iso modules drivers run test-run clean test fmt-check fmt clippy doc test-kernel install
