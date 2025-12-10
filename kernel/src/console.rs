// SPDX-License-Identifier: GPL-2.0

//! Console and kernel output

use core::fmt::{self, Write};

use crate::error::Result;
use crate::sync::Spinlock;

/// Console writer
static CONSOLE: Spinlock<Console> = Spinlock::new(Console::new());

/// VGA text mode colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
	Black = 0,
	Blue = 1,
	Green = 2,
	Cyan = 3,
	Red = 4,
	Magenta = 5,
	Brown = 6,
	LightGray = 7,
	DarkGray = 8,
	LightBlue = 9,
	LightGreen = 10,
	LightCyan = 11,
	LightRed = 12,
	Pink = 13,
	Yellow = 14,
	White = 15,
}

/// VGA text mode color code combining foreground and background colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
	const fn new(foreground: Color, background: Color) -> ColorCode {
		ColorCode((background as u8) << 4 | (foreground as u8))
	}
}

/// VGA text mode screen character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
	ascii_character: u8,
	color_code: ColorCode,
}

/// VGA text mode buffer dimensions
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// VGA text mode buffer structure
#[repr(transparent)]
struct Buffer {
	chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

struct Console {
	initialized: bool,
	vga_buffer: Option<&'static mut Buffer>,
	column_position: usize,
	color_code: ColorCode,
}

impl Console {
	const fn new() -> Self {
		Self {
			initialized: false,
			vga_buffer: None,
			column_position: 0,
			color_code: ColorCode::new(Color::Yellow, Color::Black),
		}
	}

	fn init(&mut self) -> Result<()> {
		// Initialize VGA text mode buffer
		self.vga_buffer = Some(unsafe { &mut *(0xb8000 as *mut Buffer) });

		// Initialize serial port (COM1)
		self.init_serial();

		self.clear_screen();
		self.initialized = true;
		Ok(())
	}

	fn init_serial(&self) {
		unsafe {
			// Disable interrupts
			core::arch::asm!("out dx, al", in("dx") 0x3F9u16, in("al") 0x00u8);
			// Set baud rate divisor
			core::arch::asm!("out dx, al", in("dx") 0x3FBu16, in("al") 0x80u8); // Enable DLAB
			core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") 0x03u8); // Divisor low byte (38400 baud)
			core::arch::asm!("out dx, al", in("dx") 0x3F9u16, in("al") 0x00u8); // Divisor high byte
								       // Configure line
			core::arch::asm!("out dx, al", in("dx") 0x3FBu16, in("al") 0x03u8); // 8 bits, no parity, one stop bit
			core::arch::asm!("out dx, al", in("dx") 0x3FCu16, in("al") 0xC7u8); // Enable FIFO, clear, 14-byte threshold
			core::arch::asm!("out dx, al", in("dx") 0x3FEu16, in("al") 0x0Bu8); // IRQs enabled, RTS/DSR set
		}
	}

	fn clear_screen(&mut self) {
		if let Some(ref mut buffer) = self.vga_buffer {
			let blank = ScreenChar {
				ascii_character: b' ',
				color_code: self.color_code,
			};

			for row in 0..BUFFER_HEIGHT {
				for col in 0..BUFFER_WIDTH {
					unsafe {
						core::ptr::write_volatile(
							&mut buffer.chars[row][col]
								as *mut ScreenChar,
							blank,
						);
					}
				}
			}
		}
		self.column_position = 0;
	}

	pub fn write_str(&mut self, s: &str) {
		if !self.initialized {
			return;
		}

		for byte in s.bytes() {
			match byte {
				b'\n' => self.new_line(),
				byte => {
					self.write_byte(byte);
				}
			}
		}
	}

	fn write_byte(&mut self, byte: u8) {
		// Write to serial port
		self.write_serial(byte);

		// Write to VGA buffer
		match byte {
			b'\n' => self.new_line(),
			byte => {
				if self.column_position >= BUFFER_WIDTH {
					self.new_line();
				}

				if let Some(ref mut buffer) = self.vga_buffer {
					let row = BUFFER_HEIGHT - 1;
					let col = self.column_position;
					let color_code = self.color_code;

					unsafe {
						core::ptr::write_volatile(
							&mut buffer.chars[row][col]
								as *mut ScreenChar,
							ScreenChar {
								ascii_character: byte,
								color_code,
							},
						);
					}
				}
				self.column_position += 1;
			}
		}
	}

	fn write_serial(&self, byte: u8) {
		unsafe {
			// Wait for transmit holding register to be empty
			loop {
				let mut status: u8;
				core::arch::asm!("in al, dx", out("al") status, in("dx") 0x3FDu16);
				if (status & 0x20) != 0 {
					break;
				}
			}

			// Write byte to serial port
			core::arch::asm!(
			    "out dx, al",
			    in("dx") 0x3F8u16,
			    in("al") byte,
			);
		}
	}

	fn new_line(&mut self) {
		if let Some(ref mut buffer) = self.vga_buffer {
			// Scroll up
			for row in 1..BUFFER_HEIGHT {
				for col in 0..BUFFER_WIDTH {
					unsafe {
						let character = core::ptr::read_volatile(
							&buffer.chars[row][col]
								as *const ScreenChar,
						);
						core::ptr::write_volatile(
							&mut buffer.chars[row - 1][col]
								as *mut ScreenChar,
							character,
						);
					}
				}
			}

			// Clear bottom row
			let blank = ScreenChar {
				ascii_character: b' ',
				color_code: self.color_code,
			};
			for col in 0..BUFFER_WIDTH {
				unsafe {
					core::ptr::write_volatile(
						&mut buffer.chars[BUFFER_HEIGHT - 1][col]
							as *mut ScreenChar,
						blank,
					);
				}
			}
		}
		self.column_position = 0;
	}
}

/// Initialize console
pub fn init() -> Result<()> {
	let mut console = CONSOLE.lock();
	console.init()
}

/// Print function for kernel output
pub fn _print(args: fmt::Arguments) {
	let mut console = CONSOLE.lock();
	let mut writer = ConsoleWriter(&mut *console);
	writer.write_fmt(args).unwrap();
}

/// Print function for kernel messages with prefix
pub fn _kprint(args: fmt::Arguments) {
	let mut console = CONSOLE.lock();
	let mut writer = ConsoleWriter(&mut *console);
	writer.write_fmt(args).unwrap();
}

/// Print informational message
pub fn print_info(message: &str) {
	let mut console = CONSOLE.lock();
	let mut writer = ConsoleWriter(&mut *console);
	writer.write_str("[INFO] ").unwrap();
	writer.write_str(message).unwrap();
}

/// Write string to console
pub fn write_str(s: &str) {
	let mut console = CONSOLE.lock();
	console.write_str(s);
}

/// Clear the console screen
pub fn clear() {
	let mut console = CONSOLE.lock();
	console.clear_screen();
}

struct ConsoleWriter<'a>(&'a mut Console);

impl Write for ConsoleWriter<'_> {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.0.write_str(s);
		Ok(())
	}
}
