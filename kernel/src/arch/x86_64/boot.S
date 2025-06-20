/* SPDX-License-Identifier: GPL-2.0 */
/* Multiboot header for the Rust kernel */

.section .multiboot_header
.align 8

/* Multiboot2 header */
multiboot_header_start:
    .long 0xe85250d6                /* magic number */
    .long 0                         /* architecture: i386 */
    .long multiboot_header_end - multiboot_header_start  /* header length */
    .long -(0xe85250d6 + 0 + (multiboot_header_end - multiboot_header_start))  /* checksum */

    /* Information request tag */
    .short 1                        /* type */
    .short 0                        /* flags */
    .long 12                        /* size */
    .long 6                         /* memory map */

    /* End tag */
    .short 0                        /* type */
    .short 0                        /* flags */  
    .long 8                         /* size */
multiboot_header_end:

.section .text
.global _start
.code32

_start:
    /* Set up stack */
    mov $stack_top, %esp
    
    /* Reset EFLAGS */
    pushl $0
    popf
    
    /* Check if we were loaded by multiboot */
    cmp $0x36d76289, %eax
    jne .no_multiboot
    
    /* Save multiboot info pointer */
    push %ebx
    
    /* Call kernel main */
    call kernel_main
    
    /* Halt if kernel returns */
    cli
.hang:
    hlt
    jmp .hang

.no_multiboot:
    /* No multiboot - just halt */
    cli
    hlt

.section .bss
.align 16
stack_bottom:
    .skip 16384  /* 16KB stack */
stack_top:

.section .data
multiboot_info_ptr:
    .long 0
