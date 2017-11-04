; The start file for the OS
; Does setup such as handling the jump to long mode

%define RESOLUTION_X 80
%define RESOLUTION_Y 25
%define VGA_PTR 0xb8000

global start

section .text
bits 32

start:
    
    ; Setup stack
    mov esp, stack_top
    
    ; Checks
    call check_multiboot ; Check if booted correctly
    call check_cpuid  ; Check if cpuid supported
    call check_long_mode ; Check if long mode supported
    
    ; Clear screen and print "FlowerOS boot"
    call clear_screen
    call boot_print

    hlt

; Clear the screen
; Formula: memory loc = VGA_PTR + (y * columns + x) * 2
;          => L = VGA_PTR + (y * 80 + x) * 2
; *note, the times 2 is because vga buffer is 16bit
; Columns = 80 because the res is 80x25
clear_screen:
    mov ecx, (RESOLUTION_Y * RESOLUTION_X + RESOLUTION_X) * 2 ; bottom right pixel (without video buffer pointer offset)
    
    .clear_screen_loop:

        mov eax, ecx ; copy the current count into eax
        add eax, VGA_PTR ; add the vga memory map pointer to it

        mov dword [eax], 0 ; move 2 black (null) characters into the buffer

        sub ecx, 4 ; minus 4 from ecx because 16 bit vga buffer = 2 bytes and we're clearing two pixels at once

        cmp ecx, 0 - 4 ; if eax *was* 0, it will wrap around
        jne .clear_screen_loop ; since it was't zero, jump to the top of the loop

    mov ecx, 0 ; clear ecx
    
    ret
    
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
        add ebx, VGA_PTR + 56; vga memory pointer offset = previous chars + 1 char
        
        mov [ebx], ax ; set char to code
        
        dec ecx
        
        cmp ecx, 0
        jne .print_error_code_loop ; if cx is not 0, loop
    
    push edx ; push return pointer
    ret
    

; Print out "FlowerOS boot"
boot_print:
    
    mov word [VGA_PTR +  0], 0x0246 ; F
    mov word [VGA_PTR +  2], 0x026c ; l
    mov word [VGA_PTR +  4], 0x026f ; o
    mov word [VGA_PTR +  6], 0x0277 ; w
    mov word [VGA_PTR +  8], 0x0265 ; e
    mov word [VGA_PTR + 10], 0x0272 ; r
    mov word [VGA_PTR + 12], 0x024f ; O
    mov word [VGA_PTR + 14], 0x0253 ; S
    mov word [VGA_PTR + 16], 0x0220 ;
    mov word [VGA_PTR + 18], 0x0262 ; b
    mov word [VGA_PTR + 20], 0x026f ; o
    mov word [VGA_PTR + 22], 0x026f ; o
    mov word [VGA_PTR + 24], 0x0274 ; t
    
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

 ; Stack grows the other way

stack_bottom:
    resb 1024
stack_top:
