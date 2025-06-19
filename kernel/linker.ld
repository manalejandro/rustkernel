/* SPDX-License-Identifier: GPL-2.0 */
/* Linker script for Rust Kernel x86_64 */

ENTRY(_start)

SECTIONS
{
    /* Start at 1MB (standard kernel load address) */
    . = 1M;
    
    /* Multiboot header must be early */
    .multiboot_header : {
        *(.multiboot_header)
    }
    
    /* Read-only sections */
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }
    
    /* Code section */
    .text : ALIGN(4K) {
        *(.text .text.*)
    }
    
    /* Read-write data */
    .data : ALIGN(4K) {
        *(.data .data.*)
    }
    
    /* BSS section (uninitialized data) */
    .bss : ALIGN(4K) {
        *(.bss .bss.*)
        *(COMMON)
    }
    
    /* Kernel end marker */
    __kernel_end = .;
}
