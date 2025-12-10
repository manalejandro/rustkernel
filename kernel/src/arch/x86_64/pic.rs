// SPDX-License-Identifier: GPL-2.0

//! Programmable Interrupt Controller (8259 PIC) support

use crate::arch::x86_64::port::Port;

/// Primary PIC ports
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;

/// Secondary PIC ports  
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// PIC commands
const PIC_EOI: u8 = 0x20; // End of Interrupt

/// Initialize the PIC
pub unsafe fn init_pic() {
	let mut pic1_command = Port::new(PIC1_COMMAND);
	let mut pic1_data = Port::new(PIC1_DATA);
	let mut pic2_command = Port::new(PIC2_COMMAND);
	let mut pic2_data = Port::new(PIC2_DATA);

	// Save masks
	let mask1 = pic1_data.read() as u8;
	let mask2 = pic2_data.read() as u8;

	// Initialize PIC1
	pic1_command.write(0x11); // ICW1: Initialize + expect ICW4
	io_wait();
	pic1_data.write(0x20); // ICW2: PIC1 offset (32)
	io_wait();
	pic1_data.write(0x04); // ICW3: Tell PIC1 there's a PIC2 at IRQ2
	io_wait();
	pic1_data.write(0x01); // ICW4: 8086 mode
	io_wait();

	// Initialize PIC2
	pic2_command.write(0x11); // ICW1: Initialize + expect ICW4
	io_wait();
	pic2_data.write(0x28); // ICW2: PIC2 offset (40)
	io_wait();
	pic2_data.write(0x02); // ICW3: Tell PIC2 it's at IRQ2 of PIC1
	io_wait();
	pic2_data.write(0x01); // ICW4: 8086 mode
	io_wait();

	// Restore masks
	pic1_data.write(mask1 as u32);
	pic2_data.write(mask2 as u32);
}

/// Send End of Interrupt signal
pub unsafe fn send_eoi(irq: u8) {
	let mut pic1_command = Port::new(PIC1_COMMAND);
	let mut pic2_command = Port::new(PIC2_COMMAND);

	if irq >= 8 {
		pic2_command.write(PIC_EOI as u32);
	}
	pic1_command.write(PIC_EOI as u32);
}

/// Mask (disable) an IRQ
pub unsafe fn mask_irq(irq: u8) {
	let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };

	let mut data_port = Port::new(port);
	let value = data_port.read() as u8;
	let mask = 1 << (irq % 8);
	data_port.write((value | mask) as u32);
}

/// Unmask (enable) an IRQ
pub unsafe fn unmask_irq(irq: u8) {
	let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };

	let mut data_port = Port::new(port);
	let value = data_port.read() as u8;
	let mask = 1 << (irq % 8);
	data_port.write((value & !mask) as u32);
}

/// I/O wait - small delay for old hardware
unsafe fn io_wait() {
	let mut port = Port::new(0x80);
	port.write(0);
}
