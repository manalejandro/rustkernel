# Rust Kernel

A modern, experimental x86_64 kernel written in Rust with advanced monitoring, diagnostics, and stress testing capabilities.

## Features

### Core Systems
- **Memory Management**: Page allocation, kmalloc/vmalloc, physical/virtual memory mapping
- **Process Management**: Basic process structures, kernel threads, scheduling framework
- **File System**: In-memory file system (memfs) with shell integration
- **Device Drivers**: PS/2 keyboard, serial console, basic device framework
- **Network Stack**: Basic networking with loopback interface and statistics
- **Module System**: Dynamic module loading with dependency management
- **System Calls**: Basic syscall infrastructure and user mode support

### Advanced Features
- **System Diagnostics**: Real-time health monitoring with categorized diagnostics
- **Performance Monitoring**: Comprehensive performance counters and analysis
- **Stress Testing**: Memory, CPU, and filesystem stress testing with metrics
- **Advanced Logging**: Multi-level logging with filtering and statistics
- **System Information**: Detailed hardware detection and system reporting
- **Interactive Shell**: 20+ commands for system administration and testing

### Architecture Support
- **x86_64**: Complete support with GDT, IDT, paging, and exception handling
- **Boot Process**: Multiboot-compatible with staged initialization
- **Hardware Detection**: CPUID-based CPU information and feature detection

## Quick Start

### Prerequisites
- Rust nightly toolchain
- NASM assembler
- Make
- QEMU (for testing)

### Building
```bash
# Build the kernel
make kernel

# Build in debug mode
RUSTFLAGS="-Awarnings" make kernel

# Clean build artifacts
make clean
```

### Project Structure
```
├── kernel/             # Core kernel implementation
│   ├── src/
│   │   ├── lib.rs     # Kernel entry point
│   │   ├── arch/      # Architecture-specific code (x86_64)
│   │   ├── memory/    # Memory management subsystem
│   │   ├── fs/        # File system implementation
│   │   └── ...        # Other subsystems
├── drivers/            # Device drivers
├── modules/            # Loadable kernel modules
└── src/               # Top-level crate wrapper
```

## Shell Commands

The kernel includes an interactive shell with comprehensive system administration commands:

### System Information
- `info` - Basic system information
- `sysinfo [show|compact|benchmark]` - Detailed system information
- `mem` - Memory statistics
- `uptime` - System uptime

### Diagnostics and Health
- `diag [report|check|clear|critical]` - System diagnostics
- `health [status|check|monitor]` - Health monitoring
- `stress <type> [duration]` - Stress testing (memory, cpu, filesystem, all)

### Performance Monitoring
- `perf [report|clear|counters|reset]` - Performance monitoring
- `bench [list|run|all|stress]` - Built-in benchmarks
- `log [show|clear|level|stats]` - Advanced logging

### File System
- `ls [path]` - List directory contents
- `cat <file>` - Display file contents
- `mkdir <path>` - Create directory
- `touch <file>` - Create file
- `rm <path>` - Remove file/directory

### Development
- `test [all|memory|fs|module]` - Run kernel tests
- `mod [list|test|unload]` - Module management
- `exec <program>` - Execute user programs
- `clear` - Clear screen
- `help` - Show all commands

## Key Features

### System Diagnostics
- Real-time health monitoring with automatic issue detection
- Categorized diagnostics (Memory, CPU, I/O, Network, FileSystem, Process, Kernel)
- Historical tracking with timestamps for trend analysis
- Critical issue alerts with detailed reporting

### Stress Testing
- **Memory Tests**: Rapid allocation/deallocation with leak detection
- **CPU Tests**: Intensive calculations for performance validation
- **Filesystem Tests**: File operations stress testing
- Performance metrics: operations/second, error rates, duration tracking

### Performance Monitoring
- Comprehensive performance counters
- Real-time system metrics
- Benchmark suite for system validation
- Integration with stress testing for performance analysis

### Advanced Logging
- Multi-level logging (debug, info, warn, error)
- Structured log entries with timestamps
- Filtering and search capabilities
- Statistics and analysis tools

## Development

### Architecture
The kernel follows a modular design with clear separation of concerns:

- **Core**: Essential kernel functionality and initialization
- **Memory**: Physical/virtual memory management with allocators
- **Process**: Process management and kernel threading
- **FileSystem**: VFS with in-memory implementation
- **Devices**: Driver framework with basic hardware support
- **Network**: Basic networking stack with interface management
- **Shell**: Interactive command interface for administration

### Error Handling
Robust error handling throughout with:
- Structured error types with detailed information
- Result-based error propagation
- Diagnostic integration for automatic issue tracking
- Recovery mechanisms where possible

### Testing
Comprehensive testing framework including:
- Unit tests for individual components
- Integration tests for subsystem interaction
- Stress tests for reliability validation
- Performance benchmarks for optimization

## License

SPDX-License-Identifier: GPL-2.0

This project is licensed under the GNU General Public License v2.0.

## Contributing

This is an experimental kernel project for research and educational purposes. 
Contributions are welcome through pull requests and issue reports.

## Status

**Experimental** - This kernel is in active development and not suitable for production use.
It serves as a platform for operating system research, Rust kernel development, 
and educational purposes.
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
