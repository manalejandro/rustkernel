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

## Development Status

**Experimental** - This kernel is in active development for research and educational purposes.

### Current Implementation Status

✅ **Fully Implemented**:
- Memory management (page allocation, kmalloc/vmalloc)
- Interactive shell with 20+ commands
- System diagnostics and health monitoring
- Stress testing framework
- Performance monitoring and benchmarks
- Advanced logging with filtering
- File system (in-memory)
- Device driver framework
- Module loading system
- Process/thread management
- System call infrastructure
- Network stack basics

🚧 **In Progress**:
- Full context switching
- Advanced memory features
- Enhanced device drivers
- User space integration

📋 **Planned**:
- SMP support
- Advanced file systems
- Complete TCP/IP stack
- Bootloader integration

## Contributing

This project welcomes contributions for research and educational purposes. Focus areas:
- Device driver implementations
- File system enhancements
- Network protocol development
- Performance optimizations
- Testing and validation
