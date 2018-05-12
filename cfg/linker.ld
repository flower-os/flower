ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
         /* Make sure the multiboot header comes at the beginning, and is not gc'd */
        KEEP(*(.multiboot_header))
    }

    /* Rustc emits lots of *really small* sections, so let's concatenate them */

    .text :
    {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data.rel.ro : {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        *(.bss .bss.*)
    }
}
