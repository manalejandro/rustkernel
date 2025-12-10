// SPDX-License-Identifier: GPL-2.0

//! Interrupt Descriptor Table (IDT) for x86_64

use core::mem::size_of;

use crate::arch::x86_64::port::outb;

/// IDT Entry structure for x86_64
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
	pub offset_low: u16,    // Handler function address bits 0-15
	pub selector: u16,      // Code segment selector
	pub ist: u8,            // Interrupt stack table offset
	pub type_attr: u8,      // Type and attributes
	pub offset_middle: u16, // Handler function address bits 16-31
	pub offset_high: u32,   // Handler function address bits 32-63
	pub zero: u32,          // Reserved, must be zero
}

impl IdtEntry {
	pub const fn new() -> Self {
		Self {
			offset_low: 0,
			selector: 0,
			ist: 0,
			type_attr: 0,
			offset_middle: 0,
			offset_high: 0,
			zero: 0,
		}
	}

	pub fn set_handler(&mut self, handler: extern "C" fn(), selector: u16, type_attr: u8) {
		let addr = handler as u64;

		self.offset_low = (addr & 0xFFFF) as u16;
		self.offset_middle = ((addr >> 16) & 0xFFFF) as u16;
		self.offset_high = ((addr >> 32) & 0xFFFFFFFF) as u32;

		self.selector = selector;
		self.type_attr = type_attr;
		self.ist = 0;
		self.zero = 0;
	}
}

/// IDT Pointer structure
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtPointer {
	pub limit: u16,
	pub base: u64,
}

/// IDT constants
pub const IDT_ENTRIES: usize = 256;

/// IDT type and attribute flags
pub mod type_attr {
	pub const PRESENT: u8 = 1 << 7;
	pub const RING_0: u8 = 0 << 5;
	pub const RING_1: u8 = 1 << 5;
	pub const RING_2: u8 = 2 << 5;
	pub const RING_3: u8 = 3 << 5;
	pub const INTERRUPT_GATE: u8 = 0x0E;
	pub const TRAP_GATE: u8 = 0x0F;
	pub const TASK_GATE: u8 = 0x05;
}

/// Global IDT
static mut IDT: [IdtEntry; IDT_ENTRIES] = [IdtEntry::new(); IDT_ENTRIES];

/// Exception handler stubs implemented in Rust
#[no_mangle]
pub extern "C" fn divide_error_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 0,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_divide_error(&ctx);
}

#[no_mangle]
pub extern "C" fn debug_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 1,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_debug(&ctx);
}

#[no_mangle]
pub extern "C" fn nmi_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 2,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_nmi(&ctx);
}

#[no_mangle]
pub extern "C" fn breakpoint_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 3,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_breakpoint(&ctx);
}

#[no_mangle]
pub extern "C" fn overflow_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 4,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_overflow(&ctx);
}

#[no_mangle]
pub extern "C" fn bound_range_exceeded_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 5,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_bound_range_exceeded(&ctx);
}

#[no_mangle]
pub extern "C" fn invalid_opcode_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 6,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_invalid_opcode(&ctx);
}

#[no_mangle]
pub extern "C" fn device_not_available_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 7,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_device_not_available(&ctx);
}

#[no_mangle]
pub extern "C" fn double_fault_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 8,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_double_fault(&ctx);
}

#[no_mangle]
pub extern "C" fn invalid_tss_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 10,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_invalid_tss(&ctx);
}

#[no_mangle]
pub extern "C" fn segment_not_present_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 11,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_segment_not_present(&ctx);
}

#[no_mangle]
pub extern "C" fn stack_segment_fault_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 12,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_stack_segment_fault(&ctx);
}

#[no_mangle]
pub extern "C" fn general_protection_fault_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 13,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_general_protection_fault(&ctx);
}

#[no_mangle]
pub extern "C" fn page_fault_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 14,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_page_fault(&ctx);
}

#[no_mangle]
pub extern "C" fn x87_fpu_error_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 16,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_x87_fpu_error(&ctx);
}

#[no_mangle]
pub extern "C" fn alignment_check_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 17,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_alignment_check(&ctx);
}

#[no_mangle]
pub extern "C" fn machine_check_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 18,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_machine_check(&ctx);
}

#[no_mangle]
pub extern "C" fn simd_exception_handler() {
	let ctx = ExceptionContext {
		gs: 0,
		fs: 0,
		es: 0,
		ds: 0,
		r15: 0,
		r14: 0,
		r13: 0,
		r12: 0,
		r11: 0,
		r10: 0,
		r9: 0,
		r8: 0,
		rdi: 0,
		rsi: 0,
		rbp: 0,
		rbx: 0,
		rdx: 0,
		rcx: 0,
		rax: 0,
		vector: 19,
		error_code: 0,
		rip: 0,
		cs: 0,
		eflags: 0,
		rsp: 0,
		ss: 0,
	};
	handle_simd_exception(&ctx);
}

// Hardware interrupt handlers
#[no_mangle]
pub extern "C" fn default_irq_handler() {
	// Default IRQ handler - does nothing but send EOI
	unsafe {
		crate::arch::x86_64::pic::send_eoi(0);
	}
}

// Timer interrupt handler (to be registered)
static mut TIMER_HANDLER: Option<extern "C" fn()> = None;

/// Register timer interrupt handler
pub fn register_timer_handler(handler: extern "C" fn()) {
	unsafe {
		TIMER_HANDLER = Some(handler);
		// Update IDT entry 32 (IRQ 0) with the new handler
		IDT[32].set_handler(
			timer_irq_wrapper,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
	}
}

#[no_mangle]
pub extern "C" fn timer_irq_wrapper() {
	unsafe {
		if let Some(handler) = TIMER_HANDLER {
			handler();
		} else {
			// Fallback - just send EOI
			crate::arch::x86_64::pic::send_eoi(0);
		}
	}
}

/// Initialize IDT
pub fn init() {
	unsafe {
		// Set up exception handlers
		IDT[0].set_handler(
			divide_error_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[1].set_handler(
			debug_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[2].set_handler(
			nmi_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[3].set_handler(
			breakpoint_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE | type_attr::RING_3,
		);
		IDT[4].set_handler(
			overflow_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[5].set_handler(
			bound_range_exceeded_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[6].set_handler(
			invalid_opcode_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[7].set_handler(
			device_not_available_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[8].set_handler(
			double_fault_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[10].set_handler(
			invalid_tss_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[11].set_handler(
			segment_not_present_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[12].set_handler(
			stack_segment_fault_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[13].set_handler(
			general_protection_fault_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[14].set_handler(
			page_fault_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[16].set_handler(
			x87_fpu_error_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[17].set_handler(
			alignment_check_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[18].set_handler(
			machine_check_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);
		IDT[19].set_handler(
			simd_exception_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);

		// Set up hardware interrupt handlers
		// Timer interrupt (IRQ 0 -> IDT 32)
		IDT[32].set_handler(
			default_irq_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);

		// Keyboard interrupt (IRQ 1 -> IDT 33)
		IDT[33].set_handler(
			default_irq_handler,
			0x08,
			type_attr::PRESENT | type_attr::INTERRUPT_GATE,
		);

		// Set up default handlers for other IRQs (IRQ 2-15 -> IDT 34-47)
		for i in 34..48 {
			IDT[i].set_handler(
				default_irq_handler,
				0x08,
				type_attr::PRESENT | type_attr::INTERRUPT_GATE,
			);
		}

		let idt_ptr = IdtPointer {
			limit: (size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
			base: IDT.as_ptr() as u64,
		};

		// Load IDT
		core::arch::asm!(
		    "lidt [{}]",
		    in(reg) &idt_ptr,
		    options(nostack, preserves_flags)
		);
	}
}

/// Exception context structure passed from assembly
#[derive(Debug)]
#[repr(C)]
pub struct ExceptionContext {
	// Segment registers
	pub gs: u64,
	pub fs: u64,
	pub es: u64,
	pub ds: u64,

	// General purpose registers
	pub r15: u64,
	pub r14: u64,
	pub r13: u64,
	pub r12: u64,
	pub r11: u64,
	pub r10: u64,
	pub r9: u64,
	pub r8: u64,
	pub rdi: u64,
	pub rsi: u64,
	pub rbp: u64,
	pub rbx: u64,
	pub rdx: u64,
	pub rcx: u64,
	pub rax: u64,

	// Exception information
	pub vector: u64,
	pub error_code: u64,

	// Interrupt stack frame
	pub rip: u64,
	pub cs: u64,
	pub eflags: u64,
	pub rsp: u64,
	pub ss: u64,
}

/// Exception handler called from assembly
#[no_mangle]
pub extern "C" fn exception_handler(context: *const ExceptionContext) {
	let ctx = unsafe { &*context };

	match ctx.vector {
		0 => handle_divide_error(ctx),
		1 => handle_debug(ctx),
		2 => handle_nmi(ctx),
		3 => handle_breakpoint(ctx),
		4 => handle_overflow(ctx),
		5 => handle_bound_range_exceeded(ctx),
		6 => handle_invalid_opcode(ctx),
		7 => handle_device_not_available(ctx),
		8 => handle_double_fault(ctx),
		10 => handle_invalid_tss(ctx),
		11 => handle_segment_not_present(ctx),
		12 => handle_stack_segment_fault(ctx),
		13 => handle_general_protection_fault(ctx),
		14 => handle_page_fault(ctx),
		16 => handle_x87_fpu_error(ctx),
		17 => handle_alignment_check(ctx),
		18 => handle_machine_check(ctx),
		19 => handle_simd_exception(ctx),
		_ => handle_unknown_exception(ctx),
	}
}

// Individual exception handlers
fn handle_divide_error(ctx: &ExceptionContext) {
	crate::error!("Divide by zero error at RIP: 0x{:x}", ctx.rip);
	panic!("Divide by zero exception");
}

fn handle_debug(ctx: &ExceptionContext) {
	crate::info!("Debug exception at RIP: 0x{:x}", ctx.rip);
}

fn handle_nmi(ctx: &ExceptionContext) {
	crate::error!("Non-maskable interrupt at RIP: 0x{:x}", ctx.rip);
}

fn handle_breakpoint(ctx: &ExceptionContext) {
	crate::info!("Breakpoint at RIP: 0x{:x}", ctx.rip);
}

fn handle_overflow(ctx: &ExceptionContext) {
	crate::error!("Overflow exception at RIP: 0x{:x}", ctx.rip);
	panic!("Overflow exception");
}

fn handle_bound_range_exceeded(ctx: &ExceptionContext) {
	crate::error!("Bound range exceeded at RIP: 0x{:x}", ctx.rip);
	panic!("Bound range exceeded");
}

fn handle_invalid_opcode(ctx: &ExceptionContext) {
	crate::error!("Invalid opcode at RIP: 0x{:x}", ctx.rip);
	panic!("Invalid opcode");
}

fn handle_device_not_available(ctx: &ExceptionContext) {
	crate::error!("Device not available at RIP: 0x{:x}", ctx.rip);
	panic!("Device not available");
}

fn handle_double_fault(ctx: &ExceptionContext) {
	crate::error!(
		"Double fault at RIP: 0x{:x}, error code: 0x{:x}",
		ctx.rip,
		ctx.error_code
	);
	panic!("Double fault");
}

fn handle_invalid_tss(ctx: &ExceptionContext) {
	crate::error!(
		"Invalid TSS at RIP: 0x{:x}, error code: 0x{:x}",
		ctx.rip,
		ctx.error_code
	);
	panic!("Invalid TSS");
}

fn handle_segment_not_present(ctx: &ExceptionContext) {
	crate::error!(
		"Segment not present at RIP: 0x{:x}, error code: 0x{:x}",
		ctx.rip,
		ctx.error_code
	);
	panic!("Segment not present");
}

fn handle_stack_segment_fault(ctx: &ExceptionContext) {
	crate::error!(
		"Stack segment fault at RIP: 0x{:x}, error code: 0x{:x}",
		ctx.rip,
		ctx.error_code
	);
	panic!("Stack segment fault");
}

fn handle_general_protection_fault(ctx: &ExceptionContext) {
	crate::error!(
		"General protection fault at RIP: 0x{:x}, error code: 0x{:x}",
		ctx.rip,
		ctx.error_code
	);
	panic!("General protection fault");
}

fn handle_page_fault(ctx: &ExceptionContext) {
	// Get the faulting address from CR2
	let fault_addr: u64;
	unsafe {
		core::arch::asm!("mov {}, cr2", out(reg) fault_addr);
	}

	crate::error!(
		"Page fault at RIP: 0x{:x}, fault address: 0x{:x}, error code: 0x{:x}",
		ctx.rip,
		fault_addr,
		ctx.error_code
	);
	panic!("Page fault");
}

fn handle_x87_fpu_error(ctx: &ExceptionContext) {
	crate::error!("x87 FPU error at RIP: 0x{:x}", ctx.rip);
	panic!("x87 FPU error");
}

fn handle_alignment_check(ctx: &ExceptionContext) {
	crate::error!(
		"Alignment check at RIP: 0x{:x}, error code: 0x{:x}",
		ctx.rip,
		ctx.error_code
	);
	panic!("Alignment check");
}

fn handle_machine_check(ctx: &ExceptionContext) {
	crate::error!("Machine check at RIP: 0x{:x}", ctx.rip);
	panic!("Machine check");
}

fn handle_simd_exception(ctx: &ExceptionContext) {
	crate::error!("SIMD exception at RIP: 0x{:x}", ctx.rip);
	panic!("SIMD exception");
}

fn handle_unknown_exception(ctx: &ExceptionContext) {
	crate::error!("Unknown exception {} at RIP: 0x{:x}", ctx.vector, ctx.rip);
	panic!("Unknown exception");
}

/// Initialize the PIC (Programmable Interrupt Controller)
pub fn init_pic() {
	unsafe {
		// Initialize PIC1 (master)
		outb(0x20, 0x11); // ICW1: Begin initialization
		outb(0x21, 0x20); // ICW2: IRQ0 -> INT 20h
		outb(0x21, 0x04); // ICW3: Tell PIC1 that PIC2 is at IRQ2
		outb(0x21, 0x01); // ICW4: 8086/88 (MCS-80/85) mode

		// Initialize PIC2 (slave)
		outb(0xA0, 0x11); // ICW1: Begin initialization
		outb(0xA1, 0x28); // ICW2: IRQ8 -> INT 28h
		outb(0xA1, 0x02); // ICW3: Tell PIC2 its cascade identity
		outb(0xA1, 0x01); // ICW4: 8086/88 (MCS-80/85) mode

		// Mask all interrupts on both PICs
		outb(0x21, 0xFF);
		outb(0xA1, 0xFF);
	}
}

/// Send End of Interrupt (EOI) signal to the PIC
pub fn eoi(irq: u8) {
	unsafe {
		// Send EOI signal to PIC1 or PIC2 depending on the IRQ number
		if irq >= 40 {
			// IRQ 40-47 are mapped to PIC2
			outb(0xA0, 0x20);
		}
		// Send EOI signal to PIC1
		outb(0x20, 0x20);
	}
}
