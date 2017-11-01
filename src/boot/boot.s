; The start file for the OS
; Does setup such as handling the jump to long mode

global start

section .text
bits 32

start:
    
    ; Clear screen and print "FlowerOS boot"
    call clear_screen
    call boot_print
    
    hlt

; Clear the screen
; Formula: memory loc = 0xb8000 + (y * columns + x) * 2
;          => L = 0xb8000 + (y * 80 + x) * 2
; *note, the times 2 is because vga buffer is 16bit
; Columns = 80 because the res is 80x25
clear_screen:
    mov ecx, (25 * 80 + 80) * 2 ; bottom right pixel (without video buffer pointer offset)
    
    clear_screen_loop:

        mov eax, ecx ; copy the current count into eax
        add eax, 0xb8000 ; add the vga memory map pointer to it

        mov dword [eax], 0 ; move 2 black (null) characters into the buffer

        sub ecx, 2 ; minus 2 from ecx because 16 bit vga buffer = 2 bytes

        cmp ecx, 0 - 2 ; if eax *was* 0, it will wrap around
        jne clear_screen_loop ; since it was't zero, jump to the top of the loop

    mov ecx, 0 ; clear ecx
    
    ret

; Print out "FlowerOS boot"
boot_print:
    
    mov word [0xb8000 +  0], 0x0246 ; F
    mov word [0xb8000 +  2], 0x026c ; l
    mov word [0xb8000 +  4], 0x026f ; o
    mov word [0xb8000 +  6], 0x0277 ; w
    mov word [0xb8000 +  8], 0x0265 ; e
    mov word [0xb8000 + 10], 0x0272 ; r
    mov word [0xb8000 + 12], 0x024f ; O
    mov word [0xb8000 + 14], 0x0253 ; S
    mov word [0xb8000 + 16], 0x0220 ;
    mov word [0xb8000 + 18], 0x0262 ; b
    mov word [0xb8000 + 20], 0x026f ; o
    mov word [0xb8000 + 22], 0x026f ; o
    mov word [0xb8000 + 24], 0x0274 ; t
    
    ret
