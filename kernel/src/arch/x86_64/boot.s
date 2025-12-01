# SPDX-License-Identifier: GPL-2.0
# Rust Kernel boot entry point for x86_64

.section .multiboot_header, "a"
# Multiboot 1 Header
.align 4
    .long 0x1BADB002                # magic
    .long 0x00000003                # flags (align + meminfo)
    .long -(0x1BADB002 + 0x00000003) # checksum

# Multiboot 2 Header
.align 8
header_start:
    # Multiboot2 header
    .long 0xe85250d6                # magic number
    .long 0                         # architecture (i386)
    .long header_end - header_start # header length
    # checksum
    .long -(0xe85250d6 + 0 + (header_end - header_start))
    
    # end tag
    .word 0    # type
    .word 0    # flags
    .long 8    # size
header_end:

.section .bss
# Multiboot information storage
.section .bss
multiboot_magic_store:
    .skip 4
multiboot_info_store:
    .skip 4
# Stack for the kernel
.global stack_bottom
.global stack_top
stack_bottom:
    .skip 16384  # 16 KiB stack
stack_top:

# Bootstrap page tables
.align 4096
.global boot_pml4
boot_pml4:
    .skip 4096

.global boot_pdp
boot_pdp:
    .skip 4096

.global boot_pd
boot_pd:
    .skip 4096

.section .rodata
gdt64:
    .quad 0                     # null descriptor
.set gdt64.code, . - gdt64
    .quad (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) # code segment
.set gdt64.data, . - gdt64  
    .quad (1<<44) | (1<<47) | (1<<41) # data segment
gdt64.pointer:
    .word . - gdt64 - 1         # length
    .quad gdt64                 # address

.section .text
.global _start
.code32
_start:
    # Set up stack
    movl $stack_top, %esp
    movl %esp, %ebp
    
    # Save multiboot information before we lose it or clobber EAX
    movl %eax, multiboot_magic_store
    movl %ebx, multiboot_info_store
    
    # Restore magic for check (or use stored value)
    movl multiboot_magic_store, %eax

    # Check for multiboot
    cmpl $0x36d76289, %eax
    je .multiboot_ok
    cmpl $0x2BADB002, %eax
    je .multiboot_ok
    jmp no_multiboot

.multiboot_ok:
    
    # Check for CPUID
    call check_cpuid
    test %eax, %eax
    jz no_cpuid
    
    # Check for long mode
    call check_long_mode
    test %eax, %eax
    jz no_long_mode
    
    # Set up page tables for long mode
    call setup_page_tables
    
    # Enable PAE
    movl %cr4, %eax
    orl $1 << 5, %eax  # Set PAE bit
    movl %eax, %cr4
    
    # Load page table
    movl $boot_pml4, %eax
    movl %eax, %cr3
    
    # Enable long mode
    movl $0xC0000080, %ecx  # EFER MSR
    rdmsr
    orl $1 << 8, %eax       # Set LM bit
    wrmsr
    
    # Enable paging
    movl %cr0, %eax
    orl $1 << 31, %eax      # Set PG bit
    movl %eax, %cr0
    
    # Load GDT
    lgdt gdt64.pointer
    
    # Far jump to 64-bit code
    ljmp $gdt64.code, $start64

check_cpuid:
    # Try to flip the ID bit (bit 21) in FLAGS
    pushfl
    popl %eax
    movl %eax, %ecx
    xorl $1 << 21, %eax
    pushl %eax
    popfl
    pushfl
    popl %eax
    pushl %ecx
    popfl
    cmpl %ecx, %eax
    setne %al
    movzbl %al, %eax
    ret

check_long_mode:
    # Check if extended processor info is available
    movl $0x80000000, %eax
    cpuid
    cmpl $0x80000001, %eax
    jb .no_long_mode
    
    # Check if long mode is available
    movl $0x80000001, %eax
    cpuid
    testl $1 << 29, %edx
    setnz %al
    movzbl %al, %eax
    ret
    
.no_long_mode:
    xorl %eax, %eax
    ret

setup_page_tables:
    # Map PML4[0] -> PDP
    movl $boot_pdp, %eax
    orl $0b11, %eax     # present + writable
    movl %eax, boot_pml4
    
    # Map PDP[0] -> PD
    movl $boot_pd, %eax
    orl $0b11, %eax     # present + writable
    movl %eax, boot_pdp
    
    # Map PD[0..511] -> 2MB Pages (Identity map 0-1GB)
    movl $boot_pd, %edi
    movl $0, %ebx       # Physical address
    movl $512, %ecx     # 512 entries
    
.map_pd_loop:
    movl %ebx, %eax
    orl $0b10000011, %eax # present + writable + huge (2MB)
    movl %eax, (%edi)
    addl $8, %edi       # Next entry
    addl $0x200000, %ebx # Next 2MB
    loop .map_pd_loop
    
    ret

.code64
start64:
    # Set up segment registers
    movw $gdt64.data, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    
    # Set up stack
    movq $stack_top, %rsp
    
    # Clear the screen
    call clear_screen
    
    # Print boot message
    movq $boot_msg, %rsi
    call print_string
    
    # Get multiboot parameters from saved locations
    movl multiboot_magic_store, %edi  # multiboot magic -> first argument
    movl multiboot_info_store, %esi   # multiboot info -> second argument
    
    # Call Rust kernel main with multiboot parameters
    call kernel_main_multiboot
    
    # If we get here, halt
halt:
    cli
    hlt
    jmp halt

# Clear VGA text buffer
clear_screen:
    movq $0xb8000, %rdi
    movw $0x0f20, %ax  # White on black space
    movl $2000, %ecx   # 80*25 characters
    rep stosw
    ret

# Print string to VGA buffer
# RSI = string pointer
print_string:
    movq $0xb8000, %rdi
    movb $0x0f, %ah    # White on black
.print_loop:
    lodsb
    testb %al, %al
    jz .print_done
    stosw
    jmp .print_loop
.print_done:
    ret

no_multiboot:
    # DEBUG: Print 'M' to serial port COM1
    mov $0x3f8, %dx
    mov $'M', %al
    out %al, %dx
    movl $no_multiboot_msg, %esi
    call print_string_32
    jmp halt32

no_cpuid:
    movl $no_cpuid_msg, %esi
    call print_string_32
    jmp halt32
    
no_long_mode:
    movl $no_long_mode_msg, %esi
    call print_string_32
    jmp halt32

# 32-bit string printing
print_string_32:
    movl $0xb8000, %edi
    movb $0x4f, %ah    # White on red
.print_loop_32:
    lodsb
    testb %al, %al
    jz .print_done_32
    stosw
    jmp .print_loop_32
.print_done_32:
    ret

halt32:
    cli
    hlt
    jmp halt32

.section .rodata
boot_msg:
    .asciz "Rust Kernel booting..."

no_multiboot_msg:
    .asciz "ERROR: Not loaded by multiboot bootloader"
    
no_cpuid_msg:
    .asciz "ERROR: CPUID not supported"
    
no_long_mode_msg:
    .asciz "ERROR: Long mode not supported"


