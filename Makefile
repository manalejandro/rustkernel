# SPDX-License-Identifier: GPL-2.0

# Rust Kernel Makefile
# Based on Linux kernel Rust infrastructure

RUST_KERNEL_VERSION := 0.1.0

# Build configuration
ARCH ?= x86_64
BUILD_TYPE ?= release

# Cargo configuration
CARGO := cargo
CARGO_FLAGS := --target-dir target

ifeq ($(BUILD_TYPE),debug)
    CARGO_FLAGS += 
else
    CARGO_FLAGS += --release
endif

# Kernel modules
RUST_MODULES := $(shell find modules -name "*.rs" -type f)
DRIVERS := $(shell find drivers -name "*.rs" -type f)

# Default target
all: kernel modules drivers

# Build the core kernel
kernel:
	@echo "Building Rust kernel ($(ARCH), $(BUILD_TYPE))"
	$(CARGO) build $(CARGO_FLAGS)

# Build kernel modules
modules: $(RUST_MODULES)
	@echo "Building kernel modules"
	cd modules && $(CARGO) build $(CARGO_FLAGS)

# Build drivers
drivers: $(DRIVERS)
	@echo "Building drivers"
	cd drivers && $(CARGO) build $(CARGO_FLAGS)

# Clean build artifacts
clean:
	$(CARGO) clean
	rm -rf target/

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

# Install (placeholder)
install:
	@echo "Install target not implemented yet"

.PHONY: all kernel modules drivers clean test fmt-check fmt clippy doc install
