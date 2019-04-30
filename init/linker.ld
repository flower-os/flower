ENTRY(start)

SECTIONS {

    . = 1M;

    /* Rustc emits lots of *really small* sections, so let's concatenate them */

    .text : {
        KEEP(*(.text.start))
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

    .data : {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .got : {
        *(.got .got.*)
        . = ALIGN(4K);
    }

    .bss : {
        *(COMMON)
        *(.bss .bss.*)
        . = ALIGN(4K);
    }
}
