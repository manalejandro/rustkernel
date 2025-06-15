# Rust Kernel

A modern, experimental kernel written in Rust, inspired by the Linux kernel architecture and designed for x86_64 systems.

## Overview

This project implements a basic operating system kernel in Rust, featuring:

- **Memory Management**: Page allocation, virtual memory, slab allocator, and buddy allocator
- **Process Management**: Process creation, scheduling, context switching, and signal handling
- **File System**: Virtual File System (VFS) with ramfs, procfs, and devfs implementations
- **Device Management**: Advanced device driver framework with power management
- **Network Stack**: Basic networking with interface management, ARP, and routing
- **System Calls**: Linux-compatible system call interface
- **Interrupt Handling**: x86_64 IDT setup and exception handling
- **Boot Process**: Staged hardware initialization and kernel setup

## Architecture

The kernel is organized into the following main components:

### Core Systems
- `lib.rs` - Main kernel entry point and module declarations
- `prelude.rs` - Common imports and essential types
- `error.rs` - Kernel error types and errno mappings
- `types.rs` - Fundamental kernel data types (PIDs, UIDs, device IDs, etc.)

### Memory Management (`memory/`)
- `page.rs` - Physical page frame allocation and management
- `allocator.rs` - High-level memory allocation interfaces
- `kmalloc.rs` - Kernel memory allocation (slab allocator)
- `vmalloc.rs` - Virtual memory allocation and VMA tracking
- `page_table.rs` - Page table management and virtual memory mapping

### Process Management
- `process.rs` - Process and thread structures, fork/exec/wait/exit
- `scheduler.rs` - Process scheduling and task switching
- `task.rs` - Task management and process lists

### File System (`fs/`)
- `mod.rs` - VFS core and file system registration
- `file.rs` - File descriptor management and operations
- `inode.rs` - Inode operations and metadata
- `dentry.rs` - Directory entry cache
- `super_block.rs` - File system superblock management
- `ramfs.rs` - RAM-based file system implementation
- `procfs.rs` - Process information file system
- `devfs.rs` - Device file system

### Device Management
- `device.rs` - Basic device abstraction
- `device_advanced.rs` - Advanced device driver framework with power management
- `driver.rs` - Device driver registration and management

### System Interface
- `syscall.rs` - System call dispatcher and interface
- `syscalls.rs` - Individual system call implementations

### Hardware Abstraction (`arch/x86_64/`)
- `context.rs` - CPU context switching and register management
- `port.rs` - I/O port access primitives
- `pic.rs` - Programmable Interrupt Controller setup

### Support Systems
- `sync.rs` - Synchronization primitives (spinlocks, mutexes)
- `console.rs` - VGA text mode and serial console output
- `interrupt.rs` - Interrupt handling and IDT management
- `network.rs` - Basic network stack implementation
- `boot.rs` - Hardware detection and staged kernel initialization
- `panic.rs` - Kernel panic handling

## Building

### Prerequisites

- Rust nightly toolchain
- `cargo` package manager

### Build Commands

```bash
# Build the kernel
RUSTFLAGS="-Awarnings" cargo +nightly build

# Build in release mode
RUSTFLAGS="-Awarnings" cargo +nightly build --release
```

## Features

### Memory Management
- **Physical Memory**: Buddy allocator for page frame management
- **Virtual Memory**: Page table management with identity mapping support
- **Kernel Heap**: Slab allocator for efficient small object allocation
- **Virtual Areas**: VMA tracking for memory region management

### Process Management
- **Process Creation**: `fork()` and `exec()` system calls
- **Scheduling**: Round-robin scheduler with priority support
- **Context Switching**: Full CPU state preservation and restoration
- **Signal Handling**: Basic signal delivery and handling

### File System
- **VFS Layer**: Generic file system interface
- **Multiple FS Types**: ramfs, procfs, devfs implementations
- **File Operations**: Standard POSIX-like file operations
- **Path Resolution**: Directory traversal and name lookup

### Device Drivers
- **Device Classes**: Block, character, network device categories
- **Power Management**: Device suspend/resume capabilities
- **Hot-plug Support**: Dynamic device registration and removal
- **Driver Framework**: Unified driver interface with probe/remove

### Network Stack
- **Interface Management**: Network interface abstraction
- **Protocol Support**: Ethernet, IPv4, ARP protocol handling
- **Routing**: Basic routing table and gateway support
- **Statistics**: Interface packet and byte counters

### System Calls
Linux-compatible system call interface including:
- File operations: `open`, `read`, `write`, `close`, `lseek`
- Process management: `fork`, `exec`, `wait`, `exit`, `getpid`
- Memory management: `mmap`, `munmap`, `brk`
- I/O control: `ioctl`

## Development Status

This is an experimental kernel project. Current status:

✅ **Implemented**:
- Basic kernel infrastructure and module system
- Memory management (physical and virtual)
- Process and thread management
- File system abstraction and basic implementations
- Device driver framework
- System call interface
- Interrupt handling
- Network stack basics
- Console output (VGA text + serial)

🚧 **In Progress**:
- Full context switching implementation
- Advanced memory features (copy-on-write, demand paging)
- Complete device driver implementations
- Network protocol stack completion
- User space integration

📋 **Planned**:
- Bootloader integration
- SMP (multi-core) support
- Advanced file systems (ext2, etc.)
- USB and PCI device support
- Complete POSIX compliance
- User space applications and shell

## Code Organization

The kernel follows Linux kernel conventions where applicable:

- Error handling using `Result<T, Error>` types
- Extensive use of traits for hardware abstraction
- Memory safety through Rust's ownership system
- Lock-free data structures where possible
- Modular architecture with clear component boundaries

## Safety

This kernel leverages Rust's memory safety guarantees:

- **No Buffer Overflows**: Compile-time bounds checking
- **No Use-After-Free**: Ownership system prevents dangling pointers
- **No Data Races**: Borrow checker ensures thread safety
- **Controlled Unsafe**: Unsafe blocks only where hardware interaction requires it

## Contributing

This is an educational/experimental project. Areas for contribution:

1. **Device Drivers**: Implement real hardware device drivers
2. **File Systems**: Add support for ext2, FAT32, etc.
3. **Network Protocols**: Complete TCP/IP stack implementation
4. **User Space**: Develop user space runtime and applications
5. **Testing**: Add unit tests and integration tests
6. **Documentation**: Improve code documentation and examples

## License

SPDX-License-Identifier: GPL-2.0

This project is licensed under the GNU General Public License v2.0, consistent with the Linux kernel.

## References

- [Linux Kernel Source](https://github.com/torvalds/linux)
- [OSDev Wiki](https://wiki.osdev.org/)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
- [Writing an OS in Rust](https://os.phil-opp.com/)

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     User Space                              │
├─────────────────────────────────────────────────────────────┤
│                   System Call Interface                     │
├─────────────────────────────────────────────────────────────┤
│  VFS  │ Process Mgmt │ Memory Mgmt │ Network │ Device Mgmt  │
├─────────────────────────────────────────────────────────────┤
│              Hardware Abstraction Layer (HAL)               │
├─────────────────────────────────────────────────────────────┤
│                     Hardware                                │
└─────────────────────────────────────────────────────────────┘
```

---

**Note**: This is an experimental kernel for educational purposes. It is not intended for production use.

```bash
# Build the kernel
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run clippy lints
cargo clippy -- -D warnings
```

## Features

- Memory-safe kernel implementation
- Zero-cost abstractions
- Modern async/await support for I/O operations
- Modular architecture
- Linux-compatible APIs where possible

## License

This project is licensed under GPL-2.0, following the Linux kernel license.

## Contributing

Contributions are welcome! Please follow the Linux kernel coding style and Rust conventions.
