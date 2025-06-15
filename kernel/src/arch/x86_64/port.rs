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
