// SPDX-License-Identifier: GPL-2.0

//! Serial console driver (16550 UART)

use alloc::{collections::VecDeque, string::String, vec::Vec};

use kernel::arch::x86_64::port::{inb, outb};
use kernel::device::{CharDevice, Device, DeviceType, FileOperations};
use kernel::error::{Error, Result};
use kernel::interrupt::{register_interrupt_handler, IrqHandler};
use kernel::sync::{Arc, Spinlock};

/// Standard COM port addresses
const COM1_BASE: u16 = 0x3F8;
const COM2_BASE: u16 = 0x2F8;
const COM3_BASE: u16 = 0x3E8;
const COM4_BASE: u16 = 0x2E8;

/// UART register offsets
const UART_DATA: u16 = 0; // Data register (R/W)
const UART_IER: u16 = 1; // Interrupt Enable Register
const UART_IIR: u16 = 2; // Interrupt Identification Register (R)
const UART_FCR: u16 = 2; // FIFO Control Register (W)
const UART_LCR: u16 = 3; // Line Control Register
const UART_MCR: u16 = 4; // Modem Control Register
const UART_LSR: u16 = 5; // Line Status Register
const UART_MSR: u16 = 6; // Modem Status Register
const UART_SCR: u16 = 7; // Scratch Register

/// Line Status Register bits
const LSR_DATA_READY: u8 = 0x01; // Data available
const LSR_OVERRUN_ERROR: u8 = 0x02; // Overrun error
const LSR_PARITY_ERROR: u8 = 0x04; // Parity error
const LSR_FRAMING_ERROR: u8 = 0x08; // Framing error
const LSR_BREAK_INTERRUPT: u8 = 0x10; // Break interrupt
const LSR_THR_EMPTY: u8 = 0x20; // Transmitter holding register empty
const LSR_THR_EMPTY_IDLE: u8 = 0x40; // Transmitter empty and idle
const LSR_FIFO_ERROR: u8 = 0x80; // FIFO error

/// Serial port structure
#[derive(Debug)]
pub struct SerialPort {
	/// Base I/O port address
	base: u16,
	/// Port name
	name: String,
	/// Receive buffer
	rx_buffer: VecDeque<u8>,
	/// Transmit buffer
	tx_buffer: VecDeque<u8>,
	/// Port configuration
	baudrate: u32,
	data_bits: u8,
	stop_bits: u8,
	parity: Parity,
}

/// Parity settings
#[derive(Debug, Clone, Copy)]
pub enum Parity {
	None,
	Odd,
	Even,
	Mark,
	Space,
}

impl SerialPort {
	/// Create a new serial port
	pub fn new(base: u16, name: String) -> Self {
		Self {
			base,
			name,
			rx_buffer: VecDeque::new(),
			tx_buffer: VecDeque::new(),
			baudrate: 115200,
			data_bits: 8,
			stop_bits: 1,
			parity: Parity::None,
		}
	}

	/// Initialize the serial port
	pub fn init(&mut self) -> Result<()> {
		// Disable interrupts
		unsafe {
			outb(self.base + UART_IER, 0x00);
		}

		// Set baud rate to 115200 (divisor = 1)
		unsafe {
			outb(self.base + UART_LCR, 0x80); // Enable DLAB
			outb(self.base + UART_DATA, 0x01); // Divisor low byte
			outb(self.base + UART_IER, 0x00); // Divisor high byte
		}

		// Configure line: 8 data bits, 1 stop bit, no parity
		unsafe {
			outb(self.base + UART_LCR, 0x03);
		}

		// Enable FIFO, clear them, with 14-byte threshold
		unsafe {
			outb(self.base + UART_FCR, 0xC7);
		}

		// Enable IRQs, set RTS/DSR, set AUX2 (used for interrupts)
		unsafe {
			outb(self.base + UART_MCR, 0x0B);
		}

		// Test serial chip (send 0xAE and check if serial returns same byte)
		unsafe {
			outb(self.base + UART_MCR, 0x1E); // Enable loopback mode
			outb(self.base + UART_DATA, 0xAE); // Send test byte

			if inb(self.base + UART_DATA) != 0xAE {
				return Err(Error::EIO);
			}

			// Disable loopback mode
			outb(self.base + UART_MCR, 0x0F);
		}

		// Enable interrupts
		unsafe {
			outb(self.base + UART_IER, 0x01);
		} // Enable receive interrupt

		Ok(())
	}

	/// Check if data is available for reading
	pub fn is_receive_ready(&self) -> bool {
		unsafe { (inb(self.base + UART_LSR) & LSR_DATA_READY) != 0 }
	}

	/// Check if transmitter is ready for data
	pub fn is_transmit_ready(&self) -> bool {
		unsafe { (inb(self.base + UART_LSR) & LSR_THR_EMPTY) != 0 }
	}

	/// Read a byte from the serial port (non-blocking)
	pub fn read_byte(&mut self) -> Option<u8> {
		if !self.rx_buffer.is_empty() {
			return self.rx_buffer.pop_front();
		}

		if self.is_receive_ready() {
			let byte = unsafe { inb(self.base + UART_DATA) };
			Some(byte)
		} else {
			None
		}
	}

	/// Write a byte to the serial port
	pub fn write_byte(&mut self, byte: u8) -> Result<()> {
		// Wait until transmitter is ready
		while !self.is_transmit_ready() {
			// Could yield here in a real implementation
		}

		unsafe {
			outb(self.base + UART_DATA, byte);
		}
		Ok(())
	}

	/// Write a string to the serial port
	pub fn write_str(&mut self, s: &str) -> Result<()> {
		for byte in s.bytes() {
			self.write_byte(byte)?;
		}
		Ok(())
	}

	/// Handle receive interrupt
	pub fn handle_receive_interrupt(&mut self) {
		while self.is_receive_ready() {
			let byte = unsafe { inb(self.base + UART_DATA) };
			if self.rx_buffer.len() < 1024 {
				// Prevent buffer overflow
				self.rx_buffer.push_back(byte);
			}
		}
	}
}

/// Global serial ports
static COM1: Spinlock<Option<SerialPort>> = Spinlock::new(None);

/// Serial interrupt handler
#[derive(Debug)]
pub struct SerialIrqHandler {
	port_base: u16,
}

impl SerialIrqHandler {
	pub fn new(port_base: u16) -> Self {
		Self { port_base }
	}
}

impl IrqHandler for SerialIrqHandler {
	fn handle_irq(&self, _irq: u32) -> Result<()> {
		// Handle COM1 interrupt
		if self.port_base == COM1_BASE {
			if let Some(ref mut port) = *COM1.lock() {
				port.handle_receive_interrupt();
			}
		}

		Ok(())
	}
}

/// Serial console file operations
#[derive(Debug)]
pub struct SerialConsoleOps;

impl FileOperations for SerialConsoleOps {
	fn open(
		&self,
		_inode: &kernel::device::Inode,
		_file: &mut kernel::device::File,
	) -> Result<()> {
		Ok(())
	}

	fn release(
		&self,
		_inode: &kernel::device::Inode,
		_file: &mut kernel::device::File,
	) -> Result<()> {
		Ok(())
	}

	fn read(
		&self,
		_file: &mut kernel::device::File,
		buf: &mut [u8],
		_offset: u64,
	) -> Result<usize> {
		let mut port = COM1.lock();
		if let Some(ref mut serial) = *port {
			let mut bytes_read = 0;

			while bytes_read < buf.len() {
				if let Some(byte) = serial.read_byte() {
					buf[bytes_read] = byte;
					bytes_read += 1;

					// Stop at newline
					if byte == b'\n' {
						break;
					}
				} else {
					break;
				}
			}

			Ok(bytes_read)
		} else {
			Err(Error::ENODEV)
		}
	}

	fn write(
		&self,
		_file: &mut kernel::device::File,
		buf: &[u8],
		_offset: u64,
	) -> Result<usize> {
		let mut port = COM1.lock();
		if let Some(ref mut serial) = *port {
			for &byte in buf {
				serial.write_byte(byte)?;
			}
			Ok(buf.len())
		} else {
			Err(Error::ENODEV)
		}
	}

	fn ioctl(&self, _file: &mut kernel::device::File, cmd: u32, arg: usize) -> Result<usize> {
		// Implement serial-specific ioctl commands (baudrate, etc.)
		match cmd {
			0x5401 => {
				// TCGETS - get terminal attributes
				crate::info!("Getting terminal attributes");
				Ok(0)
			}
			0x5402 => {
				// TCSETS - set terminal attributes
				crate::info!("Setting terminal attributes to {}", arg);
				Ok(0)
			}
			0x540B => {
				// TCFLSH - flush terminal I/O
				crate::info!("Flushing terminal I/O");
				self.flush();
				Ok(0)
			}
			0x5415 => {
				// TIOCGSERIAL - get serial port info
				crate::info!("Getting serial port info");
				Ok(0x3f8) // Return COM1 port address
			}
			0x541F => {
				// TIOCGPTN - get pty number (not applicable for serial)
				Err(Error::ENOTTY)
			}
			_ => {
				crate::info!("Unknown ioctl command: 0x{:x}", cmd);
				Err(Error::ENOTTY)
			}
		}
	}

	fn mmap(
		&self,
		_file: &mut kernel::device::File,
		_vma: &mut kernel::memory::VmaArea,
	) -> Result<()> {
		Err(Error::ENODEV)
	}
}

/// Initialize serial console
pub fn init() -> Result<()> {
	// Initialize COM1
	let mut com1 = SerialPort::new(COM1_BASE, "ttyS0".to_string());
	com1.init()?;

	*COM1.lock() = Some(com1);

	// Register interrupt handler for COM1 (IRQ 4)
	let handler = Arc::new(SerialIrqHandler::new(COM1_BASE));
	register_interrupt_handler(36, handler as Arc<dyn IrqHandler>)?; // IRQ 4 = INT 36

	// Create character device
	let char_dev = CharDevice::new(4, 64, 1, "ttyS0".to_string());

	Ok(())
}

/// Write to serial console (for kernel debugging)
pub fn serial_print(s: &str) {
	let mut port = COM1.lock();
	if let Some(ref mut serial) = *port {
		let _ = serial.write_str(s);
	}
}

/// Serial console macros
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::serial_print(&alloc::format!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", alloc::format!($($arg)*)));
}

/// Read a line from serial console
pub fn read_line() -> Result<String> {
	let mut line = String::new();

	loop {
		let mut port = COM1.lock();
		if let Some(ref mut serial) = *port {
			if let Some(byte) = serial.read_byte() {
				if byte == b'\r' || byte == b'\n' {
					// Echo newline
					let _ = serial.write_byte(b'\n');
					break;
				} else if byte == 8 || byte == 127 {
					// Backspace or DEL
					if !line.is_empty() {
						line.pop();
						// Echo backspace sequence
						let _ = serial.write_str("\x08 \x08");
					}
				} else if byte.is_ascii_graphic() || byte == b' ' {
					line.push(byte as char);
					// Echo character
					let _ = serial.write_byte(byte);
				}
			}
		}
		drop(port);

		// Yield CPU while waiting
		kernel::scheduler::yield_now();
	}

	Ok(line)
}
