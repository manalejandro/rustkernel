// SPDX-License-Identifier: GPL-2.0

//! Kernel main entry point

#![no_std]
#![no_main]

/// Multiboot1 header - placed at the very beginning
#[repr(C)]
#[repr(packed)]
struct MultibootHeader {
    magic: u32,
    flags: u32,
    checksum: u32,
}

/// Multiboot header must be in the first 8KB and be 4-byte aligned
#[link_section = ".multiboot_header"]
#[no_mangle]
#[used]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: 0x1BADB002,
    flags: 0x00000000,
    checksum: 0u32.wrapping_sub(0x1BADB002u32.wrapping_add(0x00000000)),
};

/// Entry point called by boot.s assembly code
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    kernel_main()
}

/// Main kernel function
fn kernel_main() -> ! {
    // Start with the simplest possible approach
    unsafe {
        let vga_buffer = 0xb8000 as *mut u16;
        
        // Clear screen
        for i in 0..80*25 {
            *vga_buffer.offset(i) = 0x0f20; // White space on black background
        }
        
        // Display kernel info
        let messages = [
            "Rust Kernel v1.0 - Successfully Booted!",
            "",
            "Available commands:",
            "  help     - Show this help",
            "  version  - Show kernel version", 
            "  clear    - Clear screen",
            "  reboot   - Restart system",
            "",
            "rustos> ",
        ];
        
        let mut row = 0;
        for message in &messages {
            let mut col = 0;
            for byte in message.bytes() {
                if col < 80 {
                    let vga_entry = (0x0f00 | byte as u16); // White text on black background
                    *vga_buffer.offset((row * 80 + col) as isize) = vga_entry;
                    col += 1;
                }
            }
            row += 1;
        }
    }
    
    // Simple command loop (just display, no real interaction yet)
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Panic handler - required for no_std
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
