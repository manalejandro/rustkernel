# Simple multiboot2 header and boot code
.section .multiboot_header, "a", @progbits
.align 8

multiboot2_header_start:
    .long 0xe85250d6                # multiboot2 magic
    .long 0                         # architecture (i386)
    .long multiboot2_header_end - multiboot2_header_start  # header length
    .long -(0xe85250d6 + 0 + (multiboot2_header_end - multiboot2_header_start))  # checksum

    # End tag
    .word 0    # type
    .word 0    # flags  
    .long 8    # size
multiboot2_header_end:

.section .text
.global _start
.code32

_start:
    # Disable interrupts
    cli
    
    # Set up a simple stack
    mov $stack_top, %esp
    mov $stack_top, %ebp
    
    # Call main Rust function
    call main
    
    # If main returns, halt
halt:
    hlt
    jmp halt

.section .bss
.align 16
stack_bottom:
    .skip 16384  # 16KB stack
stack_top:
