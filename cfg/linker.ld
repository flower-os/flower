ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
         /* Make sure the multiboot header comes at the beginning, and is not gc'd */
        KEEP(*(.multiboot_header))
    }

    .text :
    {
        *(.text)
    }
}
