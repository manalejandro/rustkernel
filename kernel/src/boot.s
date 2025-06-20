# SPDX-License-Identifier: GPL-2.0
# Multiboot header and boot code

.code32

.section .multiboot_header, "a"
.align 4

# Multiboot header
multiboot_header_start:
    .long 0x1BADB002                    # magic number (multiboot 1)
    .long 0x00000000                    # flags
    .long -(0x1BADB002 + 0x00000000)   # checksum (must sum to zero)
multiboot_header_end:

.section .text
.global _start
.type _start, @function

_start:
    # Disable interrupts
    cli
    
    # Set up a basic stack (16KB)
    mov $stack_top, %esp
    mov $stack_top, %ebp
    
    # Call Rust main function
    call rust_main
    
    # If rust_main returns, halt
halt_loop:
    hlt
    jmp halt_loop

# Reserve stack space
.section .bss
.align 16
stack_bottom:
    .skip 16384  # 16KB stack
stack_top:
