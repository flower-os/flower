ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
         /* make sure the multiboot header come at the beginning */
        *(.multiboot_header)
    }

    .text :
    {
        *(.text)
    }
}
