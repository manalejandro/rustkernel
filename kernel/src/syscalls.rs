// SPDX-License-Identifier: GPL-2.0

//! System call interface - Linux compatible

use crate::error::{Error, Result};
use crate::types::Pid;
use crate::process::{current_process, find_process, allocate_pid};

/// System call numbers (Linux compatible subset)
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum SyscallNumber {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Stat = 4,
    Fstat = 5,
    Lseek = 8,
    Mmap = 9,
    Munmap = 11,
    Brk = 12,
    Ioctl = 16,
    Access = 21,
    Pipe = 22,
    Select = 23,
    Socket = 41,
    Connect = 42,
    Accept = 43,
    Fork = 57,
    Execve = 59,
    Exit = 60,
    Wait4 = 61,
    Kill = 62,
    Getpid = 39,
    Getppid = 110,
    Getuid = 102,
    Setuid = 105,
    Getgid = 104,
    Setgid = 106,
    Gettid = 186,
    Clone = 56,
    Futex = 202,
}

/// System call arguments structure
#[derive(Debug)]
pub struct SyscallArgs {
    pub syscall_num: u64,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
}

/// System call dispatcher
pub fn handle_syscall(args: SyscallArgs) -> u64 {
    let result = match args.syscall_num {
        // Process management
        57 => sys_fork(),                                           // fork
        59 => sys_execve(args.arg0, args.arg1, args.arg2),         // execve
        60 => sys_exit(args.arg0 as i32),                          // exit
        61 => sys_wait4(args.arg0, args.arg1, args.arg2, args.arg3), // wait4
        62 => sys_kill(args.arg0 as i32, args.arg1 as i32),       // kill
        
        // Process info
        39 => Ok(sys_getpid() as u64),                            // getpid
        110 => Ok(sys_getppid() as u64),                          // getppid
        102 => Ok(sys_getuid() as u64),                           // getuid
        104 => Ok(sys_getgid() as u64),                           // getgid
        186 => Ok(sys_gettid() as u64),                           // gettid
        
        // File operations
        0 => sys_read(args.arg0 as i32, args.arg1, args.arg2),    // read
        1 => sys_write(args.arg0 as i32, args.arg1, args.arg2),   // write
        2 => sys_open(args.arg0, args.arg1 as i32, args.arg2 as u32), // open
        3 => sys_close(args.arg0 as i32),                         // close
        
        // Memory management
        9 => sys_mmap(args.arg0, args.arg1, args.arg2 as i32, args.arg3 as i32, args.arg4 as i32, args.arg5 as i64), // mmap
        11 => sys_munmap(args.arg0, args.arg1),                   // munmap
        12 => sys_brk(args.arg0),                                 // brk
        
        // Unimplemented syscalls
        _ => Err(Error::ENOSYS),
    };
    
    match result {
        Ok(value) => value,
        Err(error) => (-error.to_errno()) as u64,
    }
}

/// Process management syscalls
pub fn sys_fork() -> Result<u64> {
    // TODO: Implement fork
    // 1. Allocate new PID
    // 2. Copy current process
    // 3. Set up copy-on-write memory
    // 4. Add to scheduler
    // 5. Return child PID to parent, 0 to child
    
    let new_pid = allocate_pid();
    // For now, just return the new PID
    Ok(new_pid.0 as u64)
}

pub fn sys_execve(filename: u64, argv: u64, envp: u64) -> Result<u64> {
    // TODO: Implement execve
    // 1. Load program from filesystem
    // 2. Set up new memory layout
    // 3. Parse arguments and environment
    // 4. Set up initial stack
    // 5. Jump to entry point
    
    Err(Error::ENOSYS)
}

pub fn sys_exit(exit_code: i32) -> Result<u64> {
    // TODO: Implement exit
    // 1. Set process state to zombie
    // 2. Free resources
    // 3. Notify parent
    // 4. Schedule next process
    
    // This syscall doesn't return
    panic!("Process exit with code {}", exit_code);
}

pub fn sys_wait4(pid: u64, status: u64, options: u64, rusage: u64) -> Result<u64> {
    // TODO: Implement wait4
    // 1. Find child process
    // 2. Block until child exits
    // 3. Return child PID and status
    
    Err(Error::ECHILD)
}

pub fn sys_kill(pid: i32, signal: i32) -> Result<u64> {
    // TODO: Implement kill
    // 1. Find target process
    // 2. Send signal
    // 3. Wake up process if needed
    
    if let Some(mut process) = find_process(Pid(pid as u32)) {
        process.send_signal(signal)?;
        Ok(0)
    } else {
        Err(Error::ESRCH)
    }
}

/// Process info syscalls
pub fn sys_getpid() -> u32 {
    current_process().map(|p| p.pid.0).unwrap_or(0)
}

pub fn sys_getppid() -> u32 {
    current_process()
        .and_then(|p| p.parent)
        .map(|p| p.0)
        .unwrap_or(0)
}

pub fn sys_getuid() -> u32 {
    current_process().map(|p| p.uid.0).unwrap_or(0)
}

pub fn sys_getgid() -> u32 {
    current_process().map(|p| p.gid.0).unwrap_or(0)
}

pub fn sys_gettid() -> u32 {
    // For now, return PID (single-threaded processes)
    sys_getpid()
}

/// File operation syscalls
pub fn sys_read(fd: i32, buf: u64, count: u64) -> Result<u64> {
    // TODO: Implement read
    // 1. Get file from file descriptor table
    // 2. Read from file
    // 3. Copy to user buffer
    
    Err(Error::ENOSYS)
}

pub fn sys_write(fd: i32, buf: u64, count: u64) -> Result<u64> {
    // TODO: Implement write
    // 1. Get file from file descriptor table
    // 2. Copy from user buffer
    // 3. Write to file
    
    if fd == 1 || fd == 2 { // stdout or stderr
        // For now, just return the count as if we wrote to console
        Ok(count)
    } else {
        Err(Error::EBADF)
    }
}

pub fn sys_open(filename: u64, flags: i32, mode: u32) -> Result<u64> {
    // TODO: Implement open
    // 1. Copy filename from user space
    // 2. Open file in VFS
    // 3. Allocate file descriptor
    // 4. Add to process file table
    
    Err(Error::ENOSYS)
}

pub fn sys_close(fd: i32) -> Result<u64> {
    // TODO: Implement close
    // 1. Get file from file descriptor table
    // 2. Remove from table
    // 3. Close file
    
    Err(Error::ENOSYS)
}

/// Memory management syscalls
pub fn sys_mmap(addr: u64, length: u64, prot: i32, flags: i32, fd: i32, offset: i64) -> Result<u64> {
    // TODO: Implement mmap
    // 1. Validate parameters
    // 2. Find free virtual memory region
    // 3. Create VMA
    // 4. Set up page tables
    // 5. Return mapped address
    
    Err(Error::ENOSYS)
}

pub fn sys_munmap(addr: u64, length: u64) -> Result<u64> {
    // TODO: Implement munmap
    // 1. Find VMA containing address
    // 2. Unmap pages
    // 3. Free physical memory
    // 4. Remove VMA
    
    Err(Error::ENOSYS)
}

pub fn sys_brk(addr: u64) -> Result<u64> {
    // TODO: Implement brk
    // 1. Get current heap end
    // 2. Validate new address
    // 3. Expand or shrink heap
    // 4. Return new heap end
    
    Err(Error::ENOSYS)
}

/// Architecture-specific syscall entry point
#[cfg(target_arch = "x86_64")]
pub mod arch {
    use super::*;
    
    /// x86_64 syscall entry point (called from assembly)
    #[no_mangle]
    pub extern "C" fn syscall_entry(
        syscall_num: u64,
        arg0: u64,
        arg1: u64,
        arg2: u64,
        arg3: u64,
        arg4: u64,
        arg5: u64,
    ) -> u64 {
        let args = SyscallArgs {
            syscall_num,
            arg0,
            arg1,
            arg2,
            arg3,
            arg4,
            arg5,
        };
        
        handle_syscall(args)
    }
}

/// Initialize syscall handling
pub fn init_syscalls() -> Result<()> {
    // TODO: Set up syscall entry point in IDT/MSR
    // For x86_64, this would involve setting up the SYSCALL instruction
    
    Ok(())
}
