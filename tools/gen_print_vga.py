# Basic, top-line vga print asm code gen script


def hex_letter(letter, colour):
    return "0x0{0}{1}".format(colour, hex(ord(letter))[2:])


def gen_asm(string, colour):

    out = ""
    
    for letter, index in zip(string, range(len(string))):
    
        code = hex_letter(letter, colour)
        
        if 2 * index < 10:
            fmt_str = "mov word [0xb8000 +  {0}], {1} ; {2}\n"
        
        else:
            fmt_str = "mov word [0xb8000 + {0}], {1} ; {2}\n"
        
        out += fmt_str.format(2 * index, code, letter)
    
    return out

while True:

    print(gen_asm(input("String > "), input("Colour (Hex) > ")))
