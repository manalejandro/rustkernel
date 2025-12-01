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
		// Restore context using the pointer to self (passed in rdi)
		asm!(
			// Restore CR3 (Page Table)
			"mov rax, [rdi + 144]",
			"mov cr3, rax",

			// Switch stack to the target stack
			"mov rsp, [rdi + 56]",

			// Construct interrupt stack frame for iretq
			// Stack layout: SS, RSP, RFLAGS, CS, RIP

			// SS
			"movzx rax, word ptr [rdi + 162]",
			"push rax",

			// RSP (target stack pointer)
			"mov rax, [rdi + 56]",
			"push rax",

			// RFLAGS
			"mov rax, [rdi + 136]",
			"push rax",

			// CS
			"movzx rax, word ptr [rdi + 152]",
			"push rax",

			// RIP
			"mov rax, [rdi + 128]",
			"push rax",

			// Push General Purpose Registers onto the new stack
			// We push them in reverse order of popping
			"push qword ptr [rdi + 0]",   // rax
			"push qword ptr [rdi + 8]",   // rbx
			"push qword ptr [rdi + 16]",  // rcx
			"push qword ptr [rdi + 24]",  // rdx
			"push qword ptr [rdi + 32]",  // rsi
			"push qword ptr [rdi + 40]",  // rdi
			"push qword ptr [rdi + 48]",  // rbp
			// rsp is handled by stack switch
			"push qword ptr [rdi + 64]",  // r8
			"push qword ptr [rdi + 72]",  // r9
			"push qword ptr [rdi + 80]",  // r10
			"push qword ptr [rdi + 88]",  // r11
			"push qword ptr [rdi + 96]",  // r12
			"push qword ptr [rdi + 104]", // r13
			"push qword ptr [rdi + 112]", // r14
			"push qword ptr [rdi + 120]", // r15

			// Restore Segment Registers
			"mov ax, [rdi + 154]", // ds
			"mov ds, ax",
			"mov ax, [rdi + 156]", // es
			"mov es, ax",
			"mov ax, [rdi + 158]", // fs
			"mov fs, ax",
			"mov ax, [rdi + 160]", // gs
			"mov gs, ax",

			// Pop General Purpose Registers
			"pop r15",
			"pop r14",
			"pop r13",
			"pop r12",
			"pop r11",
			"pop r10",
			"pop r9",
			"pop r8",
			"pop rbp",
			"pop rdi", // This restores the target rdi
			"pop rsi",
			"pop rdx",
			"pop rcx",
			"pop rbx",
			"pop rax",

			// Return from interrupt (restores RIP, CS, RFLAGS, RSP, SS)
			"iretq",
			in("rdi") self,
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
