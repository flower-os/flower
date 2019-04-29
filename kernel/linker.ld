ENTRY(start)

SECTIONS {

    kernel_start = 0xffffffff80000000;
    . = kernel_start + 1M;

    .boot : AT(ADDR(.boot) - kernel_start) {
        /* Make sure the multiboot header comes at the beginning, and is not gc'd */
        KEEP(*(.multiboot_header))
        . = ALIGN(4K);
    }

    /* Rustc emits lots of *really small* sections, so let's concatenate them */

    .text : AT(ADDR(.text) - kernel_start) {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .rodata : AT(ADDR(.rodata) - kernel_start) {
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .data.rel.ro : AT(ADDR(.data.rel.ro) - kernel_start) {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

    .data : AT(ADDR(.data) - kernel_start) {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .got : AT(ADDR(.got) - kernel_start) {
        *(.got .got.*)
        . = ALIGN(4K);
    }

    .bss : AT(ADDR(.bss) - kernel_start) {
        *(COMMON)
        *(.bss .bss.*)
        . = ALIGN(4K);

        /* Make sure the guard page is before the stack and is not gc'd */
        KEEP(*(.guard_page))
        . = ALIGN(4K);

        KEEP(*(.stack))
        . = ALIGN(4K);
    }
}
