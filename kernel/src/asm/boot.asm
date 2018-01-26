; The start file for the OS
; Does setup such as handling the jump to long mode

%define RESOLUTION_X 80
%define RESOLUTION_Y 25
%define VGA_PTR 0xb8000

extern kmain
global start

section .text
bits 32

start:
    ; Disable interrupts
    cli

    ; Checks
    call check_multiboot ; Check if booted correctly
    call check_cpuid  ; Check if cpuid supported
    call check_long_mode ; Check if long mode supported
    
    ; Transition to long mode
    
    call setup_paging ; Set up paging
    
    lgdt [gdt64.pointer] ; Load gdt
        
    ; Update selector registers
    mov ax, gdt64.data ; load gdt data location into ax
    mov ss, ax ; set stack segment register
    mov ds, ax ; set data segment register
    mov es, ax ; set extra segment register
    
    jmp gdt64.code:long_mode_start
    
    hlt ; should never happen
    
; Print out error message if boot failed
; Args: length (word), ascii character codes for hex error code (words)
; Note: use push word! Otherwise it will push extra 16bit of 0s
;
; Example:
; push word 'd'
; push word 'e'
; push word 'a'
; push word 'd'
; push word 'b'
; push word 'e'
; push word 'e'
; push word 'f'
; push word 8
; call error_print
; 
; Outputs:
; FlowerOS boot failed, code 0xdeadbeef
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
    
    pop edx ; pop return pointer
    mov ecx, 0 ; clear ecx
    pop cx ; (word) length of char array
    
    .print_error_code_loop:
        
        pop ax ; pop (word) code
        or ax, 0x0400 ; form proper vga code
        
        ; ebx = vga memory location
        mov ebx, ecx ; set bx to char offset
        shl ebx, 1 ; shift for * 2 because two bits per char
        add ebx, VGA_PTR + 56; vga memory pointer offset
        
        mov [ebx], ax ; set char to code
        
        dec ecx
        
        cmp ecx, 0
        jne .print_error_code_loop ; if cx is not 0, loop
    
    push edx ; push return pointer
    ret

; Set up paging
; Thanks to https://intermezzos.github.io/book/paging.html
setup_paging:

    ; Point entry #1 of page 4 to entry #1 of page 3
    mov eax, p3_table + 0 ; set eax to 1st entry of p3 table
    or eax, 0b11
    mov [p4_table + 0], eax ; set 1st entry of p4 table to 1st entry of p3 table
    
    ; Point entry #1 of page 3 to entry #1 of page 2
    mov eax, p2_table + 0 ; set eax to 1st entry of p3 table
    or eax, 0b11
    mov [p3_table + 0], eax ; set 1st entry of p4 table to 1st entry of p3 table
    
    mov ecx, 0
    .map_p2_table_loop:
        
        mov eax, 0x200000 ; 2mib (page size) TODO 4kib page size?
        mul ecx ; multiply by counter
        or eax, 0b10000011 ; first 1 is huge page bit
        
        mov [p2_table + ecx * 8], eax
        
        inc ecx
        cmp ecx, 512
        jne .map_p2_table_loop
    
    ; Set page table address to cr3
    mov eax, p4_table ; cr3 must be mov'd to from another register
    mov cr3, eax 
    
    ; Enable Physical Address Extension
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
    
    ; Set long mode bit
    mov ecx, 0xc0000080
    rdmsr
    or eax, 1 << 8
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
    push 1
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
    push 1
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
    push 1
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

; Stack grows the other way
stack_bottom:
    resb 1024 * 16 ; 16 KiB
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
    dq gdt64 ; address of table

section .text
bits 64
long_mode_start:
    
    ; Set all data segment registers to 0
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ; Setup stack
    mov esp, stack_top
    
    call kmain
    
    hlt
