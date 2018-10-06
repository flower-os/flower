; The start file for the OS
; Does setup such as handling the jump to long mode

%define RESOLUTION_X 80
%define RESOLUTION_Y 25
%define VGA_PTR 0xb8000
%define KERNEL_MAPPING_BEGIN 0xffffffff80000000

extern kmain
global start

section .text.boot.32bit
bits 32

start:
    ; Disable interrupts
    cli

    ; Set up stack for boot section
    mov esp, stack_top - KERNEL_MAPPING_BEGIN

    ; Save multiboot2 information structure into edi to be passed into kmain
    mov edi, ebx

    ; Checks
    call check_multiboot ; Check if booted correctly
    call check_cpuid ; Check if cpuid supported
    call check_long_mode ; Check if long mode supported

    ; Transition to long mode

    call setup_paging; Set up paging

    lgdt [gdt64.pointer - KERNEL_MAPPING_BEGIN] ; Load gdt

    jmp gdt64.code:long_mode_start - KERNEL_MAPPING_BEGIN
    
; Print out error message if boot failed
; Args: length (word), ascii character codes for hex error code (words)
; Note: use push word! Otherwise it will push extra 16bit of 0s
;
; Example:
; push word 'f'
; call error_print
; 
; Outputs:
; FlowerOS boot failed, code 0xf
; 
; Error codes can be found in doc/Boot-Errors.md
error_print:
    
    mov word [VGA_PTR +  0], 0x0446 ; F
    mov word [VGA_PTR +  2], 0x046c ; l
    mov word [VGA_PTR +  4], 0x046f ; o
    mov word [VGA_PTR +  6], 0x0477 ; w
    mov word [VGA_PTR +  8], 0x0465 ; e
    mov word [VGA_PTR + 10], 0x0472 ; r
    mov word [VGA_PTR + 12], 0x044f ; O
    mov word [VGA_PTR + 14], 0x0453 ; S
    mov word [VGA_PTR + 16], 0x0420 ;
    mov word [VGA_PTR + 18], 0x0462 ; b
    mov word [VGA_PTR + 20], 0x046f ; o
    mov word [VGA_PTR + 22], 0x046f ; o
    mov word [VGA_PTR + 24], 0x0474 ; t
    mov word [VGA_PTR + 26], 0x0420 ;
    mov word [VGA_PTR + 28], 0x0466 ; f
    mov word [VGA_PTR + 30], 0x0461 ; a
    mov word [VGA_PTR + 32], 0x0469 ; i
    mov word [VGA_PTR + 34], 0x046c ; l
    mov word [VGA_PTR + 36], 0x0465 ; e
    mov word [VGA_PTR + 38], 0x0464 ; d
    mov word [VGA_PTR + 40], 0x042c ; ,
    mov word [VGA_PTR + 42], 0x0420 ;
    mov word [VGA_PTR + 44], 0x0463 ; c
    mov word [VGA_PTR + 46], 0x046f ; o
    mov word [VGA_PTR + 48], 0x0464 ; d
    mov word [VGA_PTR + 50], 0x0465 ; e
    mov word [VGA_PTR + 52], 0x0420 ;
    mov word [VGA_PTR + 54], 0x0430 ; 0
    mov word [VGA_PTR + 56], 0x0478 ; x

    pop ebx ; pop return addr
    pop ax ; pop (word) code
    or ax, 0x400 ; get vga code from character

    hlt
    mov word [VGA_PTR + 58], ax ; print the given error character

    .loop:
        hlt
        jmp .loop

; Set up paging
; Thanks to https://intermezzos.github.io/book/paging.html
setup_paging:

    ; Map p4 table
    ; Point entry #1 of page 4 to entry #1 of page 3
    mov eax, p3_table - KERNEL_MAPPING_BEGIN ; set eax to 1st entry of p3 table
    or eax, 0b11
    mov [p4_table - KERNEL_MAPPING_BEGIN], eax ; set 1st entry of p4 table to 1st entry of p3 table

    ; Map p3 table
    ; Point entry #1 of page 3 to entry #1 of page 2
    mov eax, p2_table - KERNEL_MAPPING_BEGIN ; set eax to 1st entry of p2 table
    or eax, 0b11
    mov [p3_table - KERNEL_MAPPING_BEGIN], eax ; set 1st entry of p3 table to 1st entry of p2 table

    ; Map p2 table
    mov ecx, 0
    .map_p2_table_loop:

        mov eax, 0x200000 ; 2mib (page size)
        mul ecx ; multiply by counter
        or eax, 0b10000011 ; first 1 is huge page bit

        mov ebx, p2_table - KERNEL_MAPPING_BEGIN
        mov [ebx + ecx * 8], eax

        inc ecx
        cmp ecx, 512
        jne .map_p2_table_loop

    ; Recursively map P4 table
    mov eax, p4_table - KERNEL_MAPPING_BEGIN
    or eax, 0b11 ; present & writable
    mov [p4_table - KERNEL_MAPPING_BEGIN + 510 * 8], eax

    ; Set page table address to cr3
    mov eax, p4_table - KERNEL_MAPPING_BEGIN ; cr3 must be mov'd to from another register
    mov cr3, eax

    ; Enable Physical Address Extension
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Set long mode and nxe bits
    mov ecx, 0xc0000080
    rdmsr
    or eax, 1 << 8
    or eax, 1 << 11
    wrmsr

    ; Enable paging
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    ret

; Check booted by multiboot correctly
; Thanks to Phill Opp: https://os.phil-opp.com/entering-longmode/
check_multiboot:

    ; Check if eax contains magic number 
    cmp eax, 0x36d76289
    jne .multiboot_error

    ret

; Jumped to if multiboot booted incorrectly
.multiboot_error:
    push '1' ; Error code 1
    call error_print
    hlt

; Check cpuid
; Taken from OsDev wiki: http://wiki.osdev.org/Setting_Up_Long_Mode#Detection_of_CPUID
check_cpuid:
    
    pop ebx ; pop return pointer

    ; Check if CPUID is supported by attempting to flip the ID bit (bit 21) in
    ; the FLAGS register. If we can flip it, CPUID is available.
 
    ; Copy FLAGS in to EAX via stack
    pushfd
    pop eax
 
    ; Copy to ECX as well for comparing later on
    mov ecx, eax
 
    ; Flip the ID bit
    xor eax, 1 << 21
 
    ; Copy EAX to FLAGS via the stack
    push eax
    popfd
 
    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax
 
    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit
    ; back if it was ever flipped).
    push ecx
    popfd
 
    ; Compare EAX and ECX. If they are equal then that means the bit wasn't
    ; flipped, and CPUID isn't supported.
    xor eax, ecx
    jz .no_cpuid
    
    push ebx ; push return pointer
    ret

; Jumped to if CPUID isn't supported
.no_cpuid:
    push '2' ; Error code 2
    call error_print
    hlt

; Check that the CPU supports long mode
; Taken from http://wiki.osdev.org/Setting_Up_Long_Mode#x86_or_x86-64
check_long_mode:
    mov eax, 0x80000000    ; Set the A-register to 0x80000000.
    cpuid                  ; CPU identification.
    cmp eax, 0x80000001    ; Compare the A-register with 0x80000001.
    jb .no_long_mode       ; It is less, there is no long mode.
    
    ret   

; Jumped to if long mode isn't supported
.no_long_mode:
    push '3' ; Error code 3
    call error_print
    hlt
    
section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096

p3_table_higher:
    resb 4096
p2_table_higher:
    resb 4096
p1_table_higher: ; Used for guard page area
    resb 4096

section .guard_page
align 4096

guard_page_begin:
times 4096 db 0xAA

section .stack
; Stack grows the other way
stack_bottom:
    times 1024 * 256 db 0 ; 256 kilobytes
stack_top:

section .rodata

; Copied from intermezzos: https://intermezzos.github.io/book/setting-up-a-gdt.html
gdt64:
    dq 0
.code: equ $ - gdt64 ; offset from gdt
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53)
.data: equ $ - gdt64 ; offset from gdt
    dq (1<<44) | (1<<47) | (1<<41)
.pointer:
    dw $ - gdt64 - 1 ; length
    dq gdt64 - KERNEL_MAPPING_BEGIN ; address of table

section .text.boot.64bit
bits 64
long_mode_start:

    ; Update selector registers
    mov ax, gdt64.data ; load gdt data location into ax
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Set up higher half
    call setup_higher_half

    ; Set up stack
    xor rsp, rsp
    mov rsp, stack_top

    ; Clear top 32 bits of edi
    mov rax, 0xffffffff
    and rdi, rax

    ; Pass guard page address to kmain through rsi
    mov rsi, guard_page_begin

    call kmain + KERNEL_MAPPING_BEGIN

; Set up higher half by mapping the kernel to %KERNEL_MAPPING_BEGIN
setup_higher_half:
    ; Map p4 table
    ; Point entry #511 of page 4 to entry #1 of page 3 to map at %KERNEL_MAPPING_BEGIN
    mov rax, p3_table_higher - KERNEL_MAPPING_BEGIN ; set eax to 1st entry of p3 table
    or rax, 0b11
    mov rbx, p4_table - KERNEL_MAPPING_BEGIN
    mov [rbx + 511 * 8], rax ; set 1st entry of p4 table to 1st entry of p3 table

    ; Map p3 table
    ; Point entry #510 of page 3 to entry #1 of page 2 to map at %KERNEL_MAPPING_BEGIN
    mov rax, p2_table_higher - KERNEL_MAPPING_BEGIN ; set eax to 1st entry of p2 table
    or rax, 0b11
    mov rbx, p3_table_higher - KERNEL_MAPPING_BEGIN
    mov [rbx + 510 * 8], rax ; set 1st entry of p3 table to 1st entry of p2 table

    ; Map p2 table
    mov rcx, 0
    .map_p2_table_loop:

        mov rax, 0x200000 ; 2mib (page size)
        mul rcx ; multiply by counter
        or rax, 0b10000011 ; first 1 is huge page bit
        mov rbx, p2_table_higher - KERNEL_MAPPING_BEGIN
        mov [rbx + rcx * 8], rax

        inc rcx
        cmp rcx, 512
        jne .map_p2_table_loop

    ; Reset cr3
    mov rax, cr3
    mov cr3, rax

    ret
