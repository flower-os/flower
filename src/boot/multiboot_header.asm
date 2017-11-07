; Boot headers for multiboot 2
; This lets the OS boot with any multiboot 2 compatible bootloader
; GRUB is the chosen bootloader for its ubiquity

section .multiboot_header

header_start:
    dd 0xe85250d6 ; set magic number
    dd 0 ; set protected mode
    dd header_end - header_start ; set length of the header

    ; header checksum (0x100000000 - (magic number + mode + length))
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))
    
    ; end tag
    dw 0 ; type
    dw 0 ; flags
    dd 8 ; size
header_end:
