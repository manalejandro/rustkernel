/* SPDX-License-Identifier: GPL-2.0 */
/* Linker script for Rust Kernel x86_64 */

ENTRY(_start)

/* Program headers (segments) for ELF */
PHDRS
{
    text    PT_LOAD FLAGS(5); /* Read + Execute */
    rodata  PT_LOAD FLAGS(4); /* Read only */
    data    PT_LOAD FLAGS(6); /* Read + Write */
}

SECTIONS
{
    /* Start at 1MB (standard kernel load address) */
    . = 0x100000;
    
    /* Multiboot header MUST be first */
    .multiboot_header : ALIGN(8) {
        KEEP(*(.multiboot_header))
    } :text
    
    /* Boot and text sections */
    .boot : ALIGN(8) {
        *(.text._start)
        *(.boot)
    } :text
    
    /* Rest of code section */
    .text : ALIGN(4K) {
        *(.text .text.*)
    } :text
    
    /* Read-only sections */
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    } :rodata
    
    /* Read-write data */
    .data : ALIGN(4K) {
        *(.data .data.*)
        *(.got .got.*)
    } :data
    
    /* BSS section (uninitialized data) */
    .bss : ALIGN(4K) {
        *(.bss .bss.*)
        *(COMMON)
    } :data
    
    /* Kernel end marker */
    __kernel_end = .;
}
