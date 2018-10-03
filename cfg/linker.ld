ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
         /* Make sure the multiboot header comes at the beginning, and is not gc'd */
        KEEP(*(.multiboot_header))
        . = ALIGN(4K);
    }

    /* Rustc emits lots of *really small* sections, so let's concatenate them */

    .text :
    {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .data.rel.ro : {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

    .data :
    {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .got :
    {
        *(.got .got.*)
        . = ALIGN(4K);
    }

    .bss :
    {
        *(COMMON)
        *(.bss .bss.*)
        . = ALIGN(4K);

        KEEP(*(.stack))
        . = ALIGN(4K);

        /* Make sure the guard page is before the stack and is not gc'd */
        KEEP(*(.guard_page))
        . = ALIGN(4K);
    }
}
