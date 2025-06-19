// SPDX-License-Identifier: GPL-2.0

//! Global Descriptor Table (GDT) for x86_64

use core::mem::size_of;

/// GDT Entry structure
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtEntry {
	pub limit_low: u16,
	pub base_low: u16,
	pub base_middle: u8,
	pub access: u8,
	pub granularity: u8,
	pub base_high: u8,
}

impl GdtEntry {
	pub const fn new() -> Self {
		Self {
			limit_low: 0,
			base_low: 0,
			base_middle: 0,
			access: 0,
			granularity: 0,
			base_high: 0,
		}
	}

	pub fn set_segment(&mut self, base: u32, limit: u32, access: u8, granularity: u8) {
		self.base_low = (base & 0xFFFF) as u16;
		self.base_middle = ((base >> 16) & 0xFF) as u8;
		self.base_high = ((base >> 24) & 0xFF) as u8;

		self.limit_low = (limit & 0xFFFF) as u16;
		self.granularity = ((limit >> 16) & 0x0F) as u8 | (granularity & 0xF0);

		self.access = access;
	}
}

/// GDT Pointer structure
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtPointer {
	pub limit: u16,
	pub base: u64,
}

/// GDT constants
pub const GDT_ENTRIES: usize = 5;

/// GDT access byte flags
pub mod access {
	pub const PRESENT: u8 = 1 << 7;
	pub const RING_0: u8 = 0 << 5;
	pub const RING_1: u8 = 1 << 5;
	pub const RING_2: u8 = 2 << 5;
	pub const RING_3: u8 = 3 << 5;
	pub const SYSTEM: u8 = 1 << 4;
	pub const EXECUTABLE: u8 = 1 << 3;
	pub const CONFORMING: u8 = 1 << 2;
	pub const READABLE: u8 = 1 << 1;
	pub const WRITABLE: u8 = 1 << 1;
	pub const ACCESSED: u8 = 1 << 0;
}

/// GDT granularity flags
pub mod granularity {
	pub const GRANULARITY_4K: u8 = 1 << 7;
	pub const SIZE_32: u8 = 1 << 6;
	pub const LONG_MODE: u8 = 1 << 5;
}

/// Global GDT
static mut GDT: [GdtEntry; GDT_ENTRIES] = [GdtEntry::new(); GDT_ENTRIES];

/// Initialize GDT
pub fn init() {
	unsafe {
		// Null descriptor
		GDT[0] = GdtEntry::new();

		// Kernel code segment (64-bit)
		GDT[1].set_segment(
			0x00000000,
			0xFFFFF,
			access::PRESENT
				| access::RING_0 | access::SYSTEM
				| access::EXECUTABLE | access::READABLE,
			granularity::GRANULARITY_4K | granularity::LONG_MODE,
		);

		// Kernel data segment (64-bit)
		GDT[2].set_segment(
			0x00000000,
			0xFFFFF,
			access::PRESENT | access::RING_0 | access::SYSTEM | access::WRITABLE,
			granularity::GRANULARITY_4K | granularity::LONG_MODE,
		);

		// User code segment (64-bit)
		GDT[3].set_segment(
			0x00000000,
			0xFFFFF,
			access::PRESENT
				| access::RING_3 | access::SYSTEM
				| access::EXECUTABLE | access::READABLE,
			granularity::GRANULARITY_4K | granularity::LONG_MODE,
		);

		// User data segment (64-bit)
		GDT[4].set_segment(
			0x00000000,
			0xFFFFF,
			access::PRESENT | access::RING_3 | access::SYSTEM | access::WRITABLE,
			granularity::GRANULARITY_4K | granularity::LONG_MODE,
		);

		let gdt_ptr = GdtPointer {
			limit: (size_of::<[GdtEntry; GDT_ENTRIES]>() - 1) as u16,
			base: GDT.as_ptr() as u64,
		};

		// Load GDT
		core::arch::asm!(
		    "lgdt [{}]",
		    in(reg) &gdt_ptr,
		    options(nostack, preserves_flags)
		);

		// Reload segment registers
		core::arch::asm!(
		    "mov ax, 0x10",  // Kernel data segment
		    "mov ds, ax",
		    "mov es, ax",
		    "mov fs, ax",
		    "mov gs, ax",
		    "mov ss, ax",
		    out("ax") _,
		    options(nostack, preserves_flags)
		);

		// Far jump to reload CS
		core::arch::asm!(
		    "push 0x08",     // Kernel code segment
		    "lea rax, [rip + 2f]",
		    "push rax",
		    "retfq",
		    "2:",
		    out("rax") _,
		    options(nostack, preserves_flags)
		);
	}
}
