// SPDX-License-Identifier: GPL-2.0

//! Kernel panic handler

use core::panic::PanicInfo;
use core::fmt::Write;

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
        ).ok();
    }
    
    let message = info.message();
    writeln!(writer, "Message: {}", message).ok();
    
    writeln!(writer, "===================\n").ok();
    
    // TODO: Print stack trace
    // TODO: Save panic information to log
    
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
