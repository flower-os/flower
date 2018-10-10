global setup_syscall

extern syscall_handler

bits 64

section .text
setup_syscall:
    mov rcx, 0xc0000080
    rdmsr
    or al, 1
    wrmsr

    mov rcx, 0xc0000081
    mov rdx, 0x00100008
    mov rax, 0x00000000
    wrmsr
    mov rcx, 0xc0000082
    mov rax, syscall_handler
    mov rdx, rax
    shr rdx, 32
    and rax, 0xffffffff
    wrmsr
    mov rcx, 0xc0000084
    mov rax, ~(0x202)
    xor rdx, rdx
    not rdx
    wrmsr

    ret