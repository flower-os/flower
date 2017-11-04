; The start file for the OS
; Does setup such as handling the jump to long mode

%define RESOLUTION_X 80
%define RESOLUTION_Y 25
%define VGA_PTR 0xb8000

global start

section .text
bits 32

start:
    
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
    
    clear_screen_loop:

        mov eax, ecx ; copy the current count into eax
        add eax, VGA_PTR ; add the vga memory map pointer to it

        mov dword [eax], 0 ; move 2 black (null) characters into the buffer

        sub ecx, 4 ; minus 4 from ecx because 16 bit vga buffer = 2 bytes and we're clearing two pixels at once

        cmp ecx, 0 - 4 ; if eax *was* 0, it will wrap around
        jne clear_screen_loop ; since it was't zero, jump to the top of the loop

    mov ecx, 0 ; clear ecx
    
    ret
    
; Print out error message if boot failed
; Args: length (word), ascii character codes (words)
; Note to self: USE push word! Otherwise it will push extra 16bit of 0s
; Does not return, and needs to be called with jmp
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
    mov word [VGA_PTR + 18], 0x0442 ; B
    mov word [VGA_PTR + 20], 0x046f ; o
    mov word [VGA_PTR + 22], 0x046f ; o
    mov word [VGA_PTR + 24], 0x0474 ; t
    mov word [VGA_PTR + 26], 0x0420 ;
    mov word [VGA_PTR + 28], 0x0446 ; F
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
    
    mov ecx, 0 ; clear ecx
    pop cx ; (word) length of char array
    
    print_error_code_loop:
        
        pop ax ; pop (word) code
        or ax, 0x0400 ; form proper vga code
        
        ; ebx = vga memory location
        mov ebx, ecx ; set bx to char offset
        shl ebx, 1 ; shift for * 2 because two bits per char
        add ebx, VGA_PTR + 52; vga memory pointer offset = previous chars + 1 char
        
        mov [ebx], ax ; set char to code
        
        dec ecx
        
        cmp ecx, 0
        jne print_error_code_loop ; if cx is not 0, loop
    
    hlt
    

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
