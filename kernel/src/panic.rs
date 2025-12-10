// SPDX-License-Identifier: GPL-2.0

//! Kernel panic handler

use core::fmt::Write;
use core::panic::PanicInfo;

/// Panic handler
#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
	// Disable interrupts
	#[cfg(target_arch = "x86_64")]
	unsafe {
		core::arch::asm!("cli");
	}

	// Print panic information
	let mut writer = PanicWriter;
	writeln!(writer, "\n\n=== KERNEL PANIC ===").ok();

	if let Some(location) = info.location() {
		writeln!(
			writer,
			"Panic at {}:{}:{}",
			location.file(),
			location.line(),
			location.column()
		)
		.ok();
	}

	let message = info.message();
	writeln!(writer, "Message: {}", message).ok();

	writeln!(writer, "===================\n").ok();

	// Print stack trace
	print_stack_trace(&mut writer);

	// Save panic information to system log
	save_panic_info(info);

	// Halt the system
	loop {
		#[cfg(target_arch = "x86_64")]
		unsafe {
			core::arch::asm!("hlt");
		}

		#[cfg(not(target_arch = "x86_64"))]
		core::hint::spin_loop();
	}
}

/// Print a simple stack trace
fn print_stack_trace<W: core::fmt::Write>(writer: &mut W) {
	writeln!(writer, "Stack trace:").ok();

	// Get current frame pointer
	let mut rbp: *const usize;
	unsafe {
		core::arch::asm!("mov {}, rbp", out(reg) rbp);
	}

	// Walk the stack (simplified)
	let mut frame_count = 0;
	while !rbp.is_null() && frame_count < 10 {
		unsafe {
			// Read return address from stack frame
			let ret_addr = rbp.add(1).read_volatile();
			writeln!(writer, "  #{}: 0x{:016x}", frame_count, ret_addr).ok();

			// Move to previous frame
			rbp = rbp.read_volatile() as *const usize;
			frame_count += 1;

			// Safety check to avoid infinite loops
			if (rbp as usize) < 0x1000 || (rbp as usize) > 0x7FFFFFFFFFFF {
				break;
			}
		}
	}
}

/// Save panic information to system log
fn save_panic_info(info: &core::panic::PanicInfo) {
	// In a real implementation, this would write to a persistent log
	// For now, we'll just store it in memory for potential retrieval

	if let Some(location) = info.location() {
		crate::info!(
			"PANIC LOGGED: {}:{}:{} - {}",
			location.file(),
			location.line(),
			location.column(),
			info.message()
		);
	} else {
		crate::info!("PANIC LOGGED: {}", info.message());
	}
}

/// Writer for panic messages
struct PanicWriter;

impl Write for PanicWriter {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		// Write directly to VGA buffer or serial port
		for byte in s.bytes() {
			#[cfg(target_arch = "x86_64")]
			unsafe {
				// Write to serial port (COM1)
				core::arch::asm!(
				    "out dx, al",
				    in("dx") 0x3f8u16,
				    in("al") byte,
				);
			}
		}
		Ok(())
	}
}
