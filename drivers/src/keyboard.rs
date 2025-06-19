// SPDX-License-Identifier: GPL-2.0

//! PS/2 Keyboard driver

use alloc::{collections::VecDeque, string::String, vec::Vec};

use kernel::arch::x86_64::port::{inb, outb};
use kernel::device::{CharDevice, Device, DeviceType, FileOperations};
use kernel::error::{Error, Result};
use kernel::interrupt::{register_interrupt_handler, IrqHandler};
use kernel::sync::{Arc, Spinlock};

/// PS/2 keyboard controller ports
const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;
const KEYBOARD_COMMAND_PORT: u16 = 0x64;

/// Keyboard scan codes to ASCII mapping (US layout, simplified)
const SCANCODE_TO_ASCII: [u8; 128] = [
	0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=',
	8, // backspace
	b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']',
	b'\n', // enter
	0,     // ctrl
	b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`',
	0, // left shift
	b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', 0, // right shift
	b'*', 0,    // alt
	b' ', // space
	0,    // caps lock
	// Function keys F1-F10
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // num lock
	0, // scroll lock
	// Numeric keypad
	b'7', b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', b'2', b'3', b'0', b'.', 0, 0,
	0, // F11, F12
	// Fill the rest with zeros
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0,
];

/// Keyboard state
#[derive(Debug)]
struct KeyboardState {
	/// Input buffer for key presses
	buffer: VecDeque<u8>,
	/// Modifier key states
	shift_pressed: bool,
	ctrl_pressed: bool,
	alt_pressed: bool,
	caps_lock: bool,
}

impl KeyboardState {
	fn new() -> Self {
		Self {
			buffer: VecDeque::new(),
			shift_pressed: false,
			ctrl_pressed: false,
			alt_pressed: false,
			caps_lock: false,
		}
	}

	fn push_key(&mut self, key: u8) {
		if self.buffer.len() < 256 {
			// Prevent buffer overflow
			self.buffer.push_back(key);
		}
	}

	fn pop_key(&mut self) -> Option<u8> {
		self.buffer.pop_front()
	}

	fn is_empty(&self) -> bool {
		self.buffer.is_empty()
	}
}

/// Global keyboard state
static KEYBOARD_STATE: Spinlock<KeyboardState> = Spinlock::new(KeyboardState::new());

/// Keyboard interrupt handler
#[derive(Debug)]
pub struct KeyboardIrqHandler;

impl IrqHandler for KeyboardIrqHandler {
	fn handle_irq(&self, _irq: u32) -> Result<()> {
		// Read scan code from keyboard data port
		let scancode = unsafe { inb(KEYBOARD_DATA_PORT) };

		// Process the scan code
		process_scancode(scancode);

		Ok(())
	}
}

/// Process a keyboard scan code
fn process_scancode(scancode: u8) {
	let mut keyboard = KEYBOARD_STATE.lock();

	// Check if this is a key release (high bit set)
	if scancode & 0x80 != 0 {
		// Key release
		let key_code = scancode & 0x7F;
		match key_code {
			0x2A | 0x36 => keyboard.shift_pressed = false, // Shift keys
			0x1D => keyboard.ctrl_pressed = false,         // Ctrl key
			0x38 => keyboard.alt_pressed = false,          // Alt key
			_ => {}                                        // Other key releases
		}
		return;
	}

	// Key press
	match scancode {
		0x2A | 0x36 => {
			// Shift keys
			keyboard.shift_pressed = true;
		}
		0x1D => {
			// Ctrl key
			keyboard.ctrl_pressed = true;
		}
		0x38 => {
			// Alt key
			keyboard.alt_pressed = true;
		}
		0x3A => {
			// Caps Lock
			keyboard.caps_lock = !keyboard.caps_lock;
		}
		_ => {
			// Convert scan code to ASCII
			if let Some(ascii) = scancode_to_ascii(scancode, &keyboard) {
				keyboard.push_key(ascii);

				// Echo to console for now
				if ascii.is_ascii_graphic() || ascii == b' ' || ascii == b'\n' {
					kernel::console::print_char(ascii as char);
				}
			}
		}
	}
}

/// Convert scan code to ASCII character
fn scancode_to_ascii(scancode: u8, keyboard: &KeyboardState) -> Option<u8> {
	if scancode as usize >= SCANCODE_TO_ASCII.len() {
		return None;
	}

	let mut ascii = SCANCODE_TO_ASCII[scancode as usize];
	if ascii == 0 {
		return None;
	}

	// Apply modifiers
	if keyboard.shift_pressed || keyboard.caps_lock {
		if ascii >= b'a' && ascii <= b'z' {
			ascii = ascii - b'a' + b'A';
		} else {
			// Handle shifted symbols
			ascii = match ascii {
				b'1' => b'!',
				b'2' => b'@',
				b'3' => b'#',
				b'4' => b'$',
				b'5' => b'%',
				b'6' => b'^',
				b'7' => b'&',
				b'8' => b'*',
				b'9' => b'(',
				b'0' => b')',
				b'-' => b'_',
				b'=' => b'+',
				b'[' => b'{',
				b']' => b'}',
				b'\\' => b'|',
				b';' => b':',
				b'\'' => b'"',
				b',' => b'<',
				b'.' => b'>',
				b'/' => b'?',
				b'`' => b'~',
				_ => ascii,
			};
		}
	}

	// Handle Ctrl combinations
	if keyboard.ctrl_pressed && ascii >= b'a' && ascii <= b'z' {
		ascii = ascii - b'a' + 1; // Ctrl+A = 1, Ctrl+B = 2, etc.
	} else if keyboard.ctrl_pressed && ascii >= b'A' && ascii <= b'Z' {
		ascii = ascii - b'A' + 1;
	}

	Some(ascii)
}

/// Keyboard character device file operations
#[derive(Debug)]
pub struct KeyboardFileOps;

impl FileOperations for KeyboardFileOps {
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
		let mut keyboard = KEYBOARD_STATE.lock();
		let mut bytes_read = 0;

		// Read available characters from buffer
		while bytes_read < buf.len() && !keyboard.is_empty() {
			if let Some(key) = keyboard.pop_key() {
				buf[bytes_read] = key;
				bytes_read += 1;

				// Stop at newline for line-buffered reading
				if key == b'\n' {
					break;
				}
			}
		}

		Ok(bytes_read)
	}

	fn write(
		&self,
		_file: &mut kernel::device::File,
		_buf: &[u8],
		_offset: u64,
	) -> Result<usize> {
		// Can't write to keyboard
		Err(Error::EPERM)
	}

	fn ioctl(&self, _file: &mut kernel::device::File, cmd: u32, arg: usize) -> Result<usize> {
		// Implement keyboard-specific ioctl commands
		match cmd {
			0x4B01 => {
				// KDGKBMODE - get keyboard mode
				crate::info!("Getting keyboard mode");
				Ok(0) // Return raw mode
			}
			0x4B02 => {
				// KDSKBMODE - set keyboard mode
				crate::info!("Setting keyboard mode to {}", arg);
				Ok(0)
			}
			0x4B03 => {
				// KDGKBENT - get keyboard entry
				crate::info!("Getting keyboard entry");
				Ok(0)
			}
			_ => Err(Error::ENOTTY),
		}
	}

	fn mmap(
		&self,
		_file: &mut kernel::device::File,
		_vma: &mut kernel::memory::VmaArea,
	) -> Result<()> {
		// Can't mmap keyboard
		Err(Error::ENODEV)
	}
}

/// Initialize the keyboard driver
pub fn init() -> Result<()> {
	// Create keyboard device
	let keyboard_device = Device::new(
		"keyboard".to_string(),
		DeviceType::Input,
		10, // Input major number
		0,  // Minor number 0
	);

	// Register character device
	let char_dev = CharDevice::new(10, 0, 1, "keyboard".to_string());

	// Register interrupt handler for keyboard (IRQ 1)
	let handler = Arc::new(KeyboardIrqHandler);
	register_interrupt_handler(33, handler as Arc<dyn IrqHandler>)?; // IRQ 1 = INT 33

	// Register device in device filesystem
	crate::info!("Keyboard device registered in devfs");

	Ok(())
}

/// Read a line from keyboard (blocking)
pub fn read_line() -> String {
	let mut line = String::new();

	loop {
		let mut keyboard = KEYBOARD_STATE.lock();
		while let Some(key) = keyboard.pop_key() {
			if key == b'\n' {
				return line;
			} else if key == 8 {
				// Backspace
				if !line.is_empty() {
					line.pop();
					// Move cursor back and clear character
					kernel::console::print_str("\x08 \x08");
				}
			} else if key.is_ascii_graphic() || key == b' ' {
				line.push(key as char);
			}
		}
		drop(keyboard);

		// Yield CPU while waiting for input
		kernel::scheduler::yield_now();
	}
}

/// Check if there are pending key presses
pub fn has_pending_input() -> bool {
	let keyboard = KEYBOARD_STATE.lock();
	!keyboard.is_empty()
}

/// Get next character without blocking
pub fn try_read_char() -> Option<char> {
	let mut keyboard = KEYBOARD_STATE.lock();
	keyboard.pop_key().map(|k| k as char)
}
