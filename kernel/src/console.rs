// SPDX-License-Identifier: GPL-2.0

//! Console and kernel output

use core::fmt::{self, Write};
use crate::sync::Spinlock;
use crate::error::Result;

/// Console writer
static CONSOLE: Spinlock<Console> = Spinlock::new(Console::new());

struct Console {
    initialized: bool,
}

impl Console {
    const fn new() -> Self {
        Self {
            initialized: false,
        }
    }
    
    fn init(&mut self) -> Result<()> {
        // TODO: Initialize actual console hardware
        self.initialized = true;
        Ok(())
    }
    
    fn write_str(&self, s: &str) {
        if !self.initialized {
            return;
        }
        
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
    
    fn write_byte(&self, byte: u8) {
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
}

/// Initialize console
pub fn init() -> Result<()> {
    let mut console = CONSOLE.lock();
    console.init()
}

/// Print function for kernel output
pub fn _print(args: fmt::Arguments) {
    let console = CONSOLE.lock();
    let mut writer = ConsoleWriter(&*console);
    writer.write_fmt(args).unwrap();
}

/// Print function for kernel messages with prefix
pub fn _kprint(args: fmt::Arguments) {
    let console = CONSOLE.lock();
    let mut writer = ConsoleWriter(&*console);
    writer.write_fmt(args).unwrap();
}

/// Print informational message
pub fn print_info(message: &str) {
    let console = CONSOLE.lock();
    let mut writer = ConsoleWriter(&*console);
    writer.write_str("[INFO] ").unwrap();
    writer.write_str(message).unwrap();
}

struct ConsoleWriter<'a>(&'a Console);

impl Write for ConsoleWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_str(s);
        Ok(())
    }
}
