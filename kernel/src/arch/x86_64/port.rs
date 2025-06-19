// SPDX-License-Identifier: GPL-2.0

//! Port I/O operations

/// Port I/O wrapper
pub struct Port {
    port: u16,
}

impl Port {
    pub const fn new(port: u16) -> Self {
        Self { port }
    }
    
    pub unsafe fn write(&mut self, value: u32) {
        core::arch::asm!(
            "out dx, eax",
            in("dx") self.port,
            in("eax") value,
        );
    }
    
    pub unsafe fn read(&mut self) -> u32 {
        let value: u32;
        core::arch::asm!(
            "in eax, dx",
            out("eax") value,
            in("dx") self.port,
        );
        value
    }
}

/// Read a byte from a port
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Write a byte to a port
pub unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}
