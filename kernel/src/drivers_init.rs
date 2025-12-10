// SPDX-License-Identifier: GPL-2.0

//! Driver initialization and management

use crate::error::Result;
use crate::{info, warn};

/// Initialize all built-in drivers
pub fn init_drivers() -> Result<()> {
	info!("Initializing built-in drivers");

	// Initialize keyboard driver
	init_keyboard_driver()?;

	// Initialize serial driver
	init_serial_driver()?;

	// Initialize ramdisk driver
	init_ramdisk_driver()?;

	info!("Built-in drivers initialized");
	Ok(())
}

/// Initialize PS/2 keyboard driver
fn init_keyboard_driver() -> Result<()> {
	info!("Initializing PS/2 keyboard driver");

	// Register keyboard interrupt handler (IRQ 1)
	if let Err(e) = crate::interrupt::request_irq(
		1,
		keyboard_interrupt_handler,
		0,
		"keyboard",
		core::ptr::null_mut(),
	) {
		warn!("Failed to register keyboard interrupt: {}", e);
		return Err(e);
	}

	info!("PS/2 keyboard driver initialized");
	Ok(())
}

/// Initialize serial driver
fn init_serial_driver() -> Result<()> {
	info!("Initializing serial driver");

	// Register serial interrupt handlers (IRQ 3 and 4)
	if let Err(e) = crate::interrupt::request_irq(
		3,
		serial_interrupt_handler,
		0,
		"serial",
		core::ptr::null_mut(),
	) {
		warn!("Failed to register serial interrupt: {}", e);
	}

	if let Err(e) = crate::interrupt::request_irq(
		4,
		serial_interrupt_handler,
		0,
		"serial",
		core::ptr::null_mut(),
	) {
		warn!("Failed to register serial interrupt: {}", e);
	}

	info!("Serial driver initialized");
	Ok(())
}

/// Initialize ramdisk driver
fn init_ramdisk_driver() -> Result<()> {
	info!("Initializing ramdisk driver");

	// TODO: Create ramdisk device
	// This would typically involve:
	// 1. Allocating memory for the ramdisk
	// 2. Registering the device with the block device subsystem
	// 3. Setting up device file operations

	info!("Ramdisk driver initialized");
	Ok(())
}

/// Keyboard interrupt handler
fn keyboard_interrupt_handler(irq: u32, dev_id: *mut u8) -> crate::interrupt::IrqReturn {
	// Read the scan code from the keyboard controller
	let scancode = unsafe { crate::arch::x86_64::port::inb(0x60) };

	// Convert scan code to ASCII (simplified)
	if scancode < 128 {
		let ascii = SCANCODE_TO_ASCII[scancode as usize];
		if ascii != 0 {
			// Send character to kernel shell
			if let Err(e) = crate::shell::shell_input(ascii as char) {
				crate::warn!("Failed to process shell input: {}", e);
			}
		}
	}

	crate::interrupt::IrqReturn::Handled
}

/// Serial interrupt handler
fn serial_interrupt_handler(irq: u32, dev_id: *mut u8) -> crate::interrupt::IrqReturn {
	// TODO: Handle serial port interrupts
	// This would typically involve reading from the serial port
	// and handling incoming data

	crate::interrupt::IrqReturn::Handled
}

/// Keyboard scan code to ASCII mapping (simplified US layout)
const SCANCODE_TO_ASCII: [u8; 128] = [
	0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=',
	8, // 0-14
	b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']',
	b'\n', // 15-28
	0,     // 29 ctrl
	b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', // 30-41
	0,    // 42 left shift
	b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', // 43-53
	0,    // 54 right shift
	b'*', 0,    // 55-56 alt
	b' ', // 57 space
	0,    // 58 caps lock
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 59-68 F1-F10
	0, 0, // 69-70 num lock, scroll lock
	b'7', b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', b'2', b'3', b'0',
	b'.', // 71-83 numpad
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 84-99
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 100-115
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 116-127
];
