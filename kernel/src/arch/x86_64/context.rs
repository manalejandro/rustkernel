// SPDX-License-Identifier: GPL-2.0

//! Context switching for x86_64

use core::arch::asm;

/// CPU context for x86_64
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Context {
	// General purpose registers
	pub rax: u64,
	pub rbx: u64,
	pub rcx: u64,
	pub rdx: u64,
	pub rsi: u64,
	pub rdi: u64,
	pub rbp: u64,
	pub rsp: u64,
	pub r8: u64,
	pub r9: u64,
	pub r10: u64,
	pub r11: u64,
	pub r12: u64,
	pub r13: u64,
	pub r14: u64,
	pub r15: u64,

	// Control registers
	pub rip: u64,
	pub rflags: u64,
	pub cr3: u64, // Page table base

	// Segment selectors
	pub cs: u16,
	pub ds: u16,
	pub es: u16,
	pub fs: u16,
	pub gs: u16,
	pub ss: u16,

	// FPU state (simplified)
	pub fpu_state: [u8; 512], // FXSAVE area
}

impl Context {
	pub fn new() -> Self {
		Self {
			rax: 0,
			rbx: 0,
			rcx: 0,
			rdx: 0,
			rsi: 0,
			rdi: 0,
			rbp: 0,
			rsp: 0,
			r8: 0,
			r9: 0,
			r10: 0,
			r11: 0,
			r12: 0,
			r13: 0,
			r14: 0,
			r15: 0,
			rip: 0,
			rflags: 0x200, // Enable interrupts
			cr3: 0,
			cs: 0x08, // Kernel code segment
			ds: 0x10,
			es: 0x10,
			fs: 0x10,
			gs: 0x10,
			ss: 0x10, // Kernel data segment
			fpu_state: [0; 512],
		}
	}

	/// Create a new kernel context
	pub fn new_kernel(entry_point: u64, stack_ptr: u64, page_table: u64) -> Self {
		let mut ctx = Self::new();
		ctx.rip = entry_point;
		ctx.rsp = stack_ptr;
		ctx.cr3 = page_table;
		ctx
	}

	/// Create a new user context
	pub fn new_user(entry_point: u64, stack_ptr: u64, page_table: u64) -> Self {
		let mut ctx = Self::new();
		ctx.rip = entry_point;
		ctx.rsp = stack_ptr;
		ctx.cr3 = page_table;
		ctx.cs = 0x18 | 3; // User code segment with RPL=3
		ctx.ds = 0x20 | 3; // User data segment with RPL=3
		ctx.es = 0x20 | 3;
		ctx.fs = 0x20 | 3;
		ctx.gs = 0x20 | 3;
		ctx.ss = 0x20 | 3;
		ctx.rflags |= 0x200; // Enable interrupts in user mode
		ctx
	}

	/// Save current CPU context
	pub fn save_current(&mut self) {
		unsafe {
			// Save registers in smaller groups to avoid register pressure
			asm!(
			    "mov {}, rax",
			    "mov {}, rbx",
			    "mov {}, rcx",
			    "mov {}, rdx",
			    out(reg) self.rax,
			    out(reg) self.rbx,
			    out(reg) self.rcx,
			    out(reg) self.rdx,
			);

			asm!(
			    "mov {}, rsi",
			    "mov {}, rdi",
			    "mov {}, rbp",
			    "mov {}, rsp",
			    out(reg) self.rsi,
			    out(reg) self.rdi,
			    out(reg) self.rbp,
			    out(reg) self.rsp,
			);

			asm!(
			    "mov {}, r8",
			    "mov {}, r9",
			    "mov {}, r10",
			    "mov {}, r11",
			    out(reg) self.r8,
			    out(reg) self.r9,
			    out(reg) self.r10,
			    out(reg) self.r11,
			);

			asm!(
			    "mov {}, r12",
			    "mov {}, r13",
			    "mov {}, r14",
			    "mov {}, r15",
			    out(reg) self.r12,
			    out(reg) self.r13,
			    out(reg) self.r14,
			    out(reg) self.r15,
			);

			// Save flags
			asm!("pushfq; pop {}", out(reg) self.rflags);

			// Save CR3 (page table)
			asm!("mov {}, cr3", out(reg) self.cr3);

			// Save segment registers
			asm!("mov {0:x}, cs", out(reg) self.cs);
			asm!("mov {0:x}, ds", out(reg) self.ds);
			asm!("mov {0:x}, es", out(reg) self.es);
			asm!("mov {0:x}, fs", out(reg) self.fs);
			asm!("mov {0:x}, gs", out(reg) self.gs);
			asm!("mov {0:x}, ss", out(reg) self.ss);
		}
	}

	/// Restore CPU context and switch to it
	pub unsafe fn restore(&self) -> ! {
		// For now, implement a simplified version that doesn't cause register pressure
		// TODO: Implement full context switching with proper register restoration

		// Restore page table
		asm!("mov cr3, {}", in(reg) self.cr3);

		// Set up a minimal context switch by jumping to the target RIP
		// This is a simplified version - a full implementation would restore all
		// registers
		asm!(
		    "mov rsp, {}",
		    "push {}",    // CS for iretq
		    "push {}",    // RIP for iretq
		    "pushfq",     // Push current flags
		    "pop rax",
		    "or rax, 0x200", // Enable interrupts
		    "push rax",   // RFLAGS for iretq
		    "push {}",    // CS again
		    "push {}",    // RIP again
		    "iretq",
		    in(reg) self.rsp,
		    in(reg) self.cs as u64,
		    in(reg) self.rip,
		    in(reg) self.cs as u64,
		    in(reg) self.rip,
		    options(noreturn)
		);
	}
}

/// Context switch from old context to new context
pub unsafe fn switch_context(old_ctx: &mut Context, new_ctx: &Context) {
	// Save current context
	old_ctx.save_current();

	// Restore new context
	new_ctx.restore();
}

/// Get current stack pointer
pub fn get_current_stack_pointer() -> u64 {
	let rsp: u64;
	unsafe {
		asm!("mov {}, rsp", out(reg) rsp);
	}
	rsp
}

/// Get current instruction pointer (return address)
pub fn get_current_instruction_pointer() -> u64 {
	let rip: u64;
	unsafe {
		asm!("lea {}, [rip]", out(reg) rip);
	}
	rip
}

/// Save FPU state
pub fn save_fpu_state(buffer: &mut [u8; 512]) {
	unsafe {
		asm!("fxsave [{}]", in(reg) buffer.as_mut_ptr());
	}
}

/// Restore FPU state
pub fn restore_fpu_state(buffer: &[u8; 512]) {
	unsafe {
		asm!("fxrstor [{}]", in(reg) buffer.as_ptr());
	}
}
