// SPDX-License-Identifier: GPL-2.0

//! User mode program support

use alloc::{boxed::Box, string::String, vec, vec::Vec};

use crate::arch::x86_64::context::Context;
use crate::error::{Error, Result};
use crate::memory::{PageFlags, PhysAddr, VirtAddr};
use crate::process::{Process, ProcessState, Thread};
use crate::types::{Gid, Uid};

/// User mode privilege level
pub const USER_CS: u16 = 0x1B; // GDT selector for user code segment
pub const USER_DS: u16 = 0x23; // GDT selector for user data segment

/// User mode stack size
pub const USER_STACK_SIZE: usize = 8 * 1024 * 1024; // 8MB stack

/// User mode heap start address
pub const USER_HEAP_START: u64 = 0x40000000; // 1GB

/// Simple ELF header for user programs
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SimpleElfHeader {
	pub magic: [u8; 4],
	pub class: u8,               // 32-bit or 64-bit
	pub data: u8,                // Endianness
	pub version: u8,             // ELF version
	pub entry: u64,              // Entry point
	pub program_offset: u64,     // Program header offset
	pub section_offset: u64,     // Section header offset
	pub flags: u32,              // Architecture-specific flags
	pub header_size: u16,        // ELF header size
	pub program_entry_size: u16, // Program header entry size
	pub program_count: u16,      // Number of program headers
	pub section_entry_size: u16, // Section header entry size
	pub section_count: u16,      // Number of section headers
	pub section_names: u16,      // Section header string table index
}

/// Simple program header
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ProgramHeader {
	pub type_: u32,  // Segment type
	pub flags: u32,  // Segment flags
	pub offset: u64, // Offset in file
	pub vaddr: u64,  // Virtual address
	pub paddr: u64,  // Physical address (ignored)
	pub filesz: u64, // Size in file
	pub memsz: u64,  // Size in memory
	pub align: u64,  // Alignment
}

/// User program structure
pub struct UserProgram {
	pub name: String,
	pub entry_point: u64,
	pub code: Vec<u8>,
	pub data: Vec<u8>,
	pub bss_size: usize,
}

impl UserProgram {
	/// Create a new user program
	pub fn new(name: String, code: Vec<u8>) -> Self {
		Self {
			name,
			entry_point: 0x400000, // Default entry point
			code,
			data: Vec::new(),
			bss_size: 0,
		}
	}

	/// Set entry point
	pub fn set_entry_point(mut self, entry: u64) -> Self {
		self.entry_point = entry;
		self
	}

	/// Add data section
	pub fn with_data(mut self, data: Vec<u8>) -> Self {
		self.data = data;
		self
	}

	/// Set BSS size
	pub fn with_bss_size(mut self, size: usize) -> Self {
		self.bss_size = size;
		self
	}
}

/// User mode manager
pub struct UserModeManager {
	programs: Vec<UserProgram>,
}

impl UserModeManager {
	/// Create a new user mode manager
	pub fn new() -> Self {
		Self {
			programs: Vec::new(),
		}
	}

	/// Register a user program
	pub fn register_program(&mut self, program: UserProgram) {
		crate::info!("Registering user program: {}", program.name);
		self.programs.push(program);
	}

	/// Load and execute a user program
	pub fn exec_program(&self, name: &str, args: Vec<String>) -> Result<u32> {
		// Find the program
		let program = self
			.programs
			.iter()
			.find(|p| p.name == name)
			.ok_or(Error::NotFound)?;

		crate::info!("Loading user program: {}", name);

		// Create a new process
		let pid = crate::process::allocate_pid();
		let mut process = Process::new(pid, name.into(), Uid(0), Gid(0)); // Use dummy uid/gid

		// Set up user mode address space
		self.setup_user_address_space(&mut process, program)?;

		// Create initial thread
		let tid = crate::process::allocate_tid();
		let mut thread = Thread::new(tid, pid, 0);

		// Set up user mode context
		let mut context = Context::new();
		context.rip = program.entry_point;
		context.rsp = 0x7FFFFFFFFFFF - 16; // Near top of user space
		context.cs = USER_CS;
		context.ss = USER_DS;
		context.rflags = 0x202; // Enable interrupts

		thread.context = context;
		thread.state = ProcessState::Running;

		// Add thread to process
		process.add_thread(thread);

		// Add process to process table
		let mut table = crate::process::PROCESS_TABLE.lock();
		table.add_process(process);

		// Schedule the process
		crate::scheduler::add_task(pid)?;

		crate::info!("User program {} loaded and scheduled", name);
		Ok(pid.0)
	}

	/// Set up user mode address space
	fn setup_user_address_space(
		&self,
		process: &mut Process,
		program: &UserProgram,
	) -> Result<()> {
		// Map code segment (executable)
		let code_pages = (program.code.len() + 4095) / 4096;
		for i in 0..code_pages {
			let vaddr =
				VirtAddr::new((program.entry_point + (i * 4096) as u64) as usize);
			let paddr = crate::memory::allocate_page()?;

			// Copy code data
			let src_offset = i * 4096;
			let src_len = core::cmp::min(4096, program.code.len() - src_offset);
			if src_len > 0 {
				unsafe {
					let dst = paddr.as_u64() as *mut u8;
					let src = program.code.as_ptr().add(src_offset);
					core::ptr::copy_nonoverlapping(src, dst, src_len);
				}
			}

			// Map with execute and read permissions
			crate::memory::map_page(
				vaddr,
				paddr,
				PageFlags::USER | PageFlags::PRESENT | PageFlags::EXECUTABLE,
			)?;
		}

		// Map data segment (read/write)
		if !program.data.is_empty() {
			let data_start = 0x500000; // Data starts at 5MB
			let data_pages = (program.data.len() + 4095) / 4096;

			for i in 0..data_pages {
				let vaddr =
					VirtAddr::new((data_start + (i * 4096) as u64) as usize);
				let paddr = crate::memory::allocate_page()?;

				// Copy data
				let src_offset = i * 4096;
				let src_len = core::cmp::min(4096, program.data.len() - src_offset);
				if src_len > 0 {
					unsafe {
						let dst = paddr.as_u64() as *mut u8;
						let src = program.data.as_ptr().add(src_offset);
						core::ptr::copy_nonoverlapping(src, dst, src_len);
					}
				}

				// Map with read/write permissions
				crate::memory::map_page(
					vaddr,
					paddr,
					PageFlags::USER | PageFlags::PRESENT | PageFlags::WRITABLE,
				)?;
			}
		}

		// Map BSS segment (zero-initialized)
		if program.bss_size > 0 {
			let bss_start = 0x600000; // BSS starts at 6MB
			let bss_pages = (program.bss_size + 4095) / 4096;

			for i in 0..bss_pages {
				let vaddr = VirtAddr::new((bss_start + (i * 4096) as u64) as usize);
				let paddr = crate::memory::allocate_page()?;

				// Zero-initialize
				unsafe {
					let dst = paddr.as_u64() as *mut u8;
					core::ptr::write_bytes(dst, 0, 4096);
				}

				// Map with read/write permissions
				crate::memory::map_page(
					vaddr,
					paddr,
					PageFlags::USER | PageFlags::PRESENT | PageFlags::WRITABLE,
				)?;
			}
		}

		// Map user stack
		let stack_pages = USER_STACK_SIZE / 4096;
		let stack_start = 0x7FFFFFFFF000 - USER_STACK_SIZE as u64; // Near top of user space

		for i in 0..stack_pages {
			let vaddr = VirtAddr::new((stack_start + (i * 4096) as u64) as usize);
			let paddr = crate::memory::allocate_page()?;

			// Zero-initialize stack
			unsafe {
				let dst = paddr.as_u64() as *mut u8;
				core::ptr::write_bytes(dst, 0, 4096);
			}

			// Map with read/write permissions
			crate::memory::map_page(
				vaddr,
				paddr,
				PageFlags::USER | PageFlags::PRESENT | PageFlags::WRITABLE,
			)?;
		}

		crate::info!("User address space set up for process {}", process.pid);
		Ok(())
	}

	/// List available programs
	pub fn list_programs(&self) -> Vec<&str> {
		self.programs.iter().map(|p| p.name.as_str()).collect()
	}
}

/// Global user mode manager
static mut USER_MODE_MANAGER: Option<UserModeManager> = None;
static USER_MODE_INIT: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Initialize user mode support
pub fn init_usermode() -> Result<()> {
	if USER_MODE_INIT.load(core::sync::atomic::Ordering::Acquire) {
		return Ok(());
	}

	crate::info!("Initializing user mode support");

	unsafe {
		USER_MODE_MANAGER = Some(UserModeManager::new());
	}

	// Create some simple test programs
	create_test_programs()?;

	USER_MODE_INIT.store(true, core::sync::atomic::Ordering::Release);
	crate::info!("User mode support initialized");
	Ok(())
}

/// Get the global user mode manager
pub fn get_user_mode_manager() -> Result<&'static mut UserModeManager> {
	if !USER_MODE_INIT.load(core::sync::atomic::Ordering::Acquire) {
		return Err(Error::WouldBlock);
	}

	unsafe { USER_MODE_MANAGER.as_mut().ok_or(Error::OutOfMemory) }
}

/// Create test user programs
fn create_test_programs() -> Result<()> {
	let manager = get_user_mode_manager()?;

	// Simple "hello world" program
	// This would normally be compiled user code, but for demo we'll use inline
	// assembly
	let hello_code = vec![
		// mov rax, 1      ; sys_write
		0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, // mov rdi, 1      ; stdout
		0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,
		// mov rsi, msg    ; message address (would be set at runtime)
		0x48, 0xc7, 0xc6, 0x00, 0x50, 0x40, 0x00,
		// mov rdx, 13     ; message length
		0x48, 0xc7, 0xc2, 0x0d, 0x00, 0x00, 0x00, // syscall
		0x0f, 0x05, // mov rax, 60     ; sys_exit
		0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, // mov rdi, 0      ; exit code
		0x48, 0xc7, 0xc7, 0x00, 0x00, 0x00, 0x00, // syscall
		0x0f, 0x05,
	];

	let hello_data = b"Hello, World!\n".to_vec();

	let hello_program = UserProgram::new("hello".into(), hello_code)
		.set_entry_point(0x400000)
		.with_data(hello_data);

	manager.register_program(hello_program);

	// Simple loop program (infinite loop for testing)
	let loop_code = vec![
		// loop:
		// jmp loop
		0xeb, 0xfe,
	];

	let loop_program = UserProgram::new("loop".into(), loop_code).set_entry_point(0x400000);

	manager.register_program(loop_program);

	crate::info!("Test user programs created");
	Ok(())
}

/// Execute a user program
pub fn exec_user_program(name: &str, args: Vec<String>) -> Result<u32> {
	let manager = get_user_mode_manager()?;
	manager.exec_program(name, args)
}

/// List available user programs
pub fn list_user_programs() -> Result<Vec<&'static str>> {
	let manager = get_user_mode_manager()?;
	Ok(manager.list_programs())
}
