# SPDX-License-Identifier: GPL-2.0
# Kernel entry point - multiboot compliant

.section .multiboot
.align 4

# Multiboot header
multiboot_header:
    .long 0x1BADB002                # Magic number
    .long 0x00000003                # Flags (align modules on page boundaries + memory info)
    .long -(0x1BADB002 + 0x00000003) # Checksum

.section .bss
.align 16
stack_bottom:
    .skip 16384                     # 16 KB stack
stack_top:

.section .text
.global _start
.type _start, @function

_start:
    # Set up the stack
    mov $stack_top, %esp
    
    # Reset EFLAGS
    pushl $0
    popf
    
    # Push multiboot parameters
    pushl %ebx                      # Multiboot info structure
    pushl %eax                      # Multiboot magic number
    
    # Call the kernel main function
    call kernel_main_multiboot
    
    # If kernel returns (shouldn't happen), hang
    cli
hang:
    hlt
    jmp hang

.size _start, . - _start
