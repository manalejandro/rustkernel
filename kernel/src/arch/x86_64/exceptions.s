# SPDX-License-Identifier: GPL-2.0
# Exception handler stubs for x86_64

.section .text

# Macro for exception handlers without error code
.macro EXCEPTION_STUB name, vector
.global \name
\name:
    push $0                 # Push dummy error code
    push $\vector           # Push vector number
    jmp exception_common
.endm

# Macro for exception handlers with error code
.macro EXCEPTION_STUB_ERR name, vector
.global \name
\name:
    push $\vector           # Push vector number (error code already on stack)
    jmp exception_common
.endm

# Exception handlers
EXCEPTION_STUB divide_error_handler, 0
EXCEPTION_STUB debug_handler, 1
EXCEPTION_STUB nmi_handler, 2
EXCEPTION_STUB breakpoint_handler, 3
EXCEPTION_STUB overflow_handler, 4
EXCEPTION_STUB bound_range_exceeded_handler, 5
EXCEPTION_STUB invalid_opcode_handler, 6
EXCEPTION_STUB device_not_available_handler, 7
EXCEPTION_STUB_ERR double_fault_handler, 8
EXCEPTION_STUB_ERR invalid_tss_handler, 10
EXCEPTION_STUB_ERR segment_not_present_handler, 11
EXCEPTION_STUB_ERR stack_segment_fault_handler, 12
EXCEPTION_STUB_ERR general_protection_fault_handler, 13
EXCEPTION_STUB_ERR page_fault_handler, 14
EXCEPTION_STUB x87_fpu_error_handler, 16
EXCEPTION_STUB_ERR alignment_check_handler, 17
EXCEPTION_STUB machine_check_handler, 18
EXCEPTION_STUB simd_exception_handler, 19

# Common exception handler
exception_common:
    # Save all registers
    push %rax
    push %rcx
    push %rdx
    push %rbx
    push %rbp
    push %rsi
    push %rdi
    push %r8
    push %r9
    push %r10
    push %r11
    push %r12
    push %r13
    push %r14
    push %r15
    
    # Save segment registers
    mov %ds, %ax
    push %rax
    mov %es, %ax
    push %rax
    mov %fs, %ax
    push %rax
    mov %gs, %ax
    push %rax
    
    # Load kernel data segment
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    
    # Call exception handler
    mov %rsp, %rdi          # Pass stack pointer
    call exception_handler
    
    # Restore segment registers
    pop %rax
    mov %ax, %gs
    pop %rax
    mov %ax, %fs
    pop %rax
    mov %ax, %es
    pop %rax
    mov %ax, %ds
    
    # Restore all registers
    pop %r15
    pop %r14
    pop %r13
    pop %r12
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rdi
    pop %rsi
    pop %rbp
    pop %rbx
    pop %rdx
    pop %rcx
    pop %rax
    
    # Remove vector number and error code
    add $16, %rsp
    
    # Return from interrupt
    iretq
