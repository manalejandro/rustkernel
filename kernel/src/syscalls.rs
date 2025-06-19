// SPDX-License-Identifier: GPL-2.0

//! System call interface - Linux compatible

use crate::error::{Error, Result};
use crate::process::{allocate_pid, current_process, find_process};
use crate::types::Pid;

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
		57 => sys_fork(),                                            // fork
		59 => sys_execve(args.arg0, args.arg1, args.arg2),           // execve
		60 => sys_exit(args.arg0 as i32),                            // exit
		61 => sys_wait4(args.arg0, args.arg1, args.arg2, args.arg3), // wait4
		62 => sys_kill(args.arg0 as i32, args.arg1 as i32),          // kill

		// Process info
		39 => Ok(sys_getpid() as u64),   // getpid
		110 => Ok(sys_getppid() as u64), // getppid
		102 => Ok(sys_getuid() as u64),  // getuid
		104 => Ok(sys_getgid() as u64),  // getgid
		186 => Ok(sys_gettid() as u64),  // gettid

		// File operations
		0 => sys_read(args.arg0 as i32, args.arg1, args.arg2), // read
		1 => sys_write(args.arg0 as i32, args.arg1, args.arg2), // write
		2 => sys_open(args.arg0, args.arg1 as i32, args.arg2 as u32), // open
		3 => sys_close(args.arg0 as i32),                      // close

		// Memory management
		9 => sys_mmap(
			args.arg0,
			args.arg1,
			args.arg2 as i32,
			args.arg3 as i32,
			args.arg4 as i32,
			args.arg5 as i64,
		), // mmap
		11 => sys_munmap(args.arg0, args.arg1), // munmap
		12 => sys_brk(args.arg0),               // brk

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
	use crate::process::create_process;
	use crate::scheduler::add_task;

	// Get current process
	let current = current_process().ok_or(Error::ESRCH)?;

	// Fork the process
	let child = current.fork()?;
	let child_pid = child.pid;

	// Add child to process table and scheduler
	let mut table = crate::process::PROCESS_TABLE.lock();
	table.add_process(child.clone());
	drop(table);

	// Add to scheduler
	add_task(child_pid)?;

	// Return child PID to parent (in child, this would return 0)
	Ok(child_pid.0 as u64)
}

pub fn sys_execve(filename: u64, argv: u64, envp: u64) -> Result<u64> {
	use crate::memory::{copy_string_from_user, UserPtr};

	// Copy filename from user space
	let user_ptr = UserPtr::from_const(filename as *const u8)?;
	let filename_str = copy_string_from_user(user_ptr, 256)?;

	// Get current process
	let mut current = current_process().ok_or(Error::ESRCH)?;

	// Execute new program (with empty args for now)
	current.exec(&filename_str, alloc::vec![])?;

	// This doesn't return on success
	Ok(0)
}

pub fn sys_exit(exit_code: i32) -> Result<u64> {
	use crate::scheduler::remove_task;

	// Get current process
	if let Some(mut current) = current_process() {
		// Set exit code and mark as zombie
		current.exit(exit_code);

		// Remove from scheduler
		let _ = remove_task(current.pid);

		// In a real implementation, this would:
		// 1. Free all process resources
		// 2. Notify parent process
		// 3. Reparent children to init
		// 4. Schedule next process

		// Signal scheduler to switch to next process
		crate::scheduler::schedule();
	}

	// This syscall doesn't return
	loop {
		unsafe { core::arch::asm!("hlt") };
	}
}

pub fn sys_wait4(pid: u64, status: u64, options: u64, rusage: u64) -> Result<u64> {
	use crate::memory::{copy_to_user, UserPtr};

	// Get current process
	let current = current_process().ok_or(Error::ESRCH)?;

	// Wait for child process
	let (child_pid, exit_status) = current.wait()?;

	// If status pointer is provided, write exit status
	if status != 0 {
		let status_ptr = UserPtr::new(status as *mut i32)?;
		copy_to_user(status_ptr.cast(), &exit_status.to_ne_bytes())?;
	}

	Ok(child_pid.0 as u64)
}

pub fn sys_kill(pid: i32, signal: i32) -> Result<u64> {
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
	use crate::fs::{get_file_descriptor, read_file};
	use crate::memory::{copy_to_user, UserPtr};

	// Validate parameters
	if count == 0 {
		return Ok(0);
	}

	// Get file from file descriptor table
	let file = get_file_descriptor(fd).ok_or(Error::EBADF)?;

	// Create a kernel buffer to read into
	let mut kernel_buf = alloc::vec![0u8; count as usize];

	// Read from file
	let bytes_read = read_file(&file, &mut kernel_buf)?;

	// Copy to user buffer
	let user_ptr = UserPtr::new(buf as *mut u8)?;
	copy_to_user(user_ptr, &kernel_buf[..bytes_read])?;

	Ok(bytes_read as u64)
}

pub fn sys_write(fd: i32, buf: u64, count: u64) -> Result<u64> {
	use crate::fs::{get_file_descriptor, write_file};
	use crate::memory::{copy_from_user, UserPtr};

	// Validate parameters
	if count == 0 {
		return Ok(0);
	}

	// Handle stdout/stderr specially for now
	if fd == 1 || fd == 2 {
		// Create kernel buffer and copy from user
		let mut kernel_buf = alloc::vec![0u8; count as usize];
		let user_ptr = UserPtr::from_const(buf as *const u8)?;
		copy_from_user(&mut kernel_buf, user_ptr)?;

		// Write to console (for debugging)
		if let Ok(s) = core::str::from_utf8(&kernel_buf) {
			crate::print!("{}", s);
		}

		return Ok(count);
	}

	// Get file from file descriptor table
	let file = get_file_descriptor(fd).ok_or(Error::EBADF)?;

	// Create kernel buffer and copy from user
	let mut kernel_buf = alloc::vec![0u8; count as usize];
	let user_ptr = UserPtr::from_const(buf as *const u8)?;
	copy_from_user(&mut kernel_buf, user_ptr)?;

	// Write to file
	let bytes_written = write_file(&file, &kernel_buf)?;

	Ok(bytes_written as u64)
}

pub fn sys_open(filename: u64, flags: i32, mode: u32) -> Result<u64> {
	use crate::fs::{allocate_file_descriptor, open_file};
	use crate::memory::{copy_string_from_user, UserPtr};

	// Copy filename from user space
	let user_ptr = UserPtr::from_const(filename as *const u8)?;
	let filename_str = copy_string_from_user(user_ptr, 256)?; // Max 256 chars

	// Open file in VFS
	let file = open_file(&filename_str, flags, mode)?;

	// Allocate file descriptor and add to process file table
	let fd = allocate_file_descriptor(file)?;

	Ok(fd as u64)
}

pub fn sys_close(fd: i32) -> Result<u64> {
	use crate::fs::close_file_descriptor;

	// Close file descriptor
	close_file_descriptor(fd)?;

	Ok(0)
}

/// Memory management syscalls
pub fn sys_mmap(
	addr: u64,
	length: u64,
	prot: i32,
	flags: i32,
	fd: i32,
	offset: i64,
) -> Result<u64> {
	use crate::memory::{allocate_virtual_memory, VirtAddr, VmaArea};

	// Validate parameters
	if length == 0 {
		return Err(Error::EINVAL);
	}

	// Align length to page boundary
	let page_size = 4096u64;
	let aligned_length = (length + page_size - 1) & !(page_size - 1);

	// Allocate virtual memory region
	let vma = if addr == 0 {
		// Let kernel choose address
		allocate_virtual_memory(aligned_length, prot as u32, flags as u32)?
	} else {
		// Use specified address (with validation)
		let virt_addr = VirtAddr::new(addr as usize);

		// Validate that the address range is available
		if !crate::memory::is_virtual_range_free(virt_addr, aligned_length) {
			return Err(Error::EEXIST);
		}

		// Allocate and map the memory
		let vma = allocate_virtual_memory(aligned_length, prot as u32, flags as u32)?;
		vma
	};

	// Handle file mapping
	if fd >= 0 {
		// Map file into memory - simplified implementation
		crate::info!("File mapping requested for fd {} at offset {}", fd, offset);
		// In a real implementation, this would:
		// 1. Get the file descriptor from current process
		// 2. Map file pages into the VMA
		// 3. Set up page fault handler for demand paging
	}

	Ok(vma.vm_start.as_usize() as u64)
}

pub fn sys_munmap(addr: u64, length: u64) -> Result<u64> {
	use crate::memory::{free_virtual_memory, VirtAddr};

	// Validate parameters
	if length == 0 {
		return Err(Error::EINVAL);
	}

	// Align to page boundaries
	let page_size = 4096u64;
	let aligned_addr = addr & !(page_size - 1);
	let aligned_length = (length + page_size - 1) & !(page_size - 1);

	// Free virtual memory region
	free_virtual_memory(VirtAddr::new(aligned_addr as usize), aligned_length)?;

	Ok(0)
}

pub fn sys_brk(addr: u64) -> Result<u64> {
	use crate::memory::{get_heap_end, set_heap_end, VirtAddr};

	// Get current heap end
	let current_brk = get_heap_end();

	if addr == 0 {
		// Return current heap end
		return Ok(current_brk.as_usize() as u64);
	}

	let new_brk = VirtAddr::new(addr as usize);

	// Validate new address
	if new_brk < current_brk {
		// Shrinking heap - free pages
		let pages_to_free = (current_brk.as_usize() - new_brk.as_usize() + 4095) / 4096;
		crate::info!("Shrinking heap, freeing {} pages", pages_to_free);
		// In a real implementation, this would free the actual pages
	} else if new_brk > current_brk {
		// Expanding heap - allocate pages
		let pages_to_alloc = (new_brk.as_usize() - current_brk.as_usize() + 4095) / 4096;
		crate::info!("Expanding heap, allocating {} pages", pages_to_alloc);
		// In a real implementation, this would allocate and map new
		// pages
	}

	// Update heap end
	set_heap_end(new_brk)?;

	Ok(new_brk.as_usize() as u64)
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
	// Set up syscall entry point for x86_64
	#[cfg(target_arch = "x86_64")]
	unsafe {
		// Enable SYSCALL/SYSRET instructions
		// Set up STAR MSR (syscall target address register)
		let star_msr = 0xC0000081u32;
		let lstar_msr = 0xC0000082u32;
		let sfmask_msr = 0xC0000084u32;

		// Set up kernel and user code segments in STAR
		// Format: [63:48] user CS, [47:32] kernel CS
		let star_value = (0x1Bu64 << 48) | (0x08u64 << 32);

		// Write STAR MSR
		core::arch::asm!(
		    "wrmsr",
		    in("ecx") star_msr,
		    in("eax") (star_value & 0xFFFFFFFF) as u32,
		    in("edx") (star_value >> 32) as u32,
		    options(nostack, preserves_flags)
		);

		// Set LSTAR to point to our syscall entry
		let entry_addr = arch::syscall_entry as *const () as u64;
		core::arch::asm!(
		    "wrmsr",
		    in("ecx") lstar_msr,
		    in("eax") (entry_addr & 0xFFFFFFFF) as u32,
		    in("edx") (entry_addr >> 32) as u32,
		    options(nostack, preserves_flags)
		);

		// Set SFMASK to mask interrupt flag during syscall
		core::arch::asm!(
		    "wrmsr",
		    in("ecx") sfmask_msr,
		    in("eax") 0x200u32, // Mask IF (interrupt flag)
		    in("edx") 0u32,
		    options(nostack, preserves_flags)
		);

		// Enable SCE (System Call Extensions) in EFER
		let efer_msr = 0xC0000080u32;
		let mut efer_low: u32;
		let mut efer_high: u32;

		// Read current EFER
		core::arch::asm!(
		    "rdmsr",
		    in("ecx") efer_msr,
		    out("eax") efer_low,
		    out("edx") efer_high,
		    options(nostack, preserves_flags)
		);

		// Set SCE bit (bit 0)
		efer_low |= 1;

		// Write back EFER
		core::arch::asm!(
		    "wrmsr",
		    in("ecx") efer_msr,
		    in("eax") efer_low,
		    in("edx") efer_high,
		    options(nostack, preserves_flags)
		);
	}

	crate::info!("Syscall handling initialized");
	Ok(())
}
