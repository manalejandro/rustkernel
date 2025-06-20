.section .multiboot_header
.align 4

.long 0x1BADB002    /* magic */
.long 0x00000000    /* flags */
.long -(0x1BADB002 + 0x00000000)  /* checksum */

.section .text
.global _start
_start:
    /* Call Rust main function */
    call rust_main
    
    /* If rust_main returns, halt */
1:  hlt
    jmp 1b
