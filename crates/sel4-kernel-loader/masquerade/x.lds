/*
 * Copyright 2026, Colias Group, LLC
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

ENTRY(_start)

PHDRS
{
    all PT_LOAD AT (0);
}

SECTIONS
{
    . = 0;

    .text : {
        KEEP(*(.text.header))
        *(.text*)
    } :all

    .rodata : {
        *(.rodata*)
    } :all

    .rela.dyn : {
        . = ALIGN(8);
        __rela_start = .;
        KEEP(*(.rela.dyn))
        __rela_end = .;
    } :all

    .data.rel.ro : {
        *(.data.rel.ro .data.rel.ro.*)
    } :all

    .got : {
        *(.got)
    } :all

    . = ALIGN(8);

    _payload_start = .;

    .symtab             0 : { *(.symtab) }
    .strtab             0 : { *(.strtab) }
    .shstrtab           0 : { *(.shstrtab) }

    .debug_abbrev       0 : { *(.debug_abbrev*) }
    .debug_info         0 : { *(.debug_info*) }
    .debug_aranges      0 : { *(.debug_aranges*) }
    .debug_ranges       0 : { *(.debug_ranges*) }
    .debug_rnglists     0 : { *(.debug_rnglists*) }
    .debug_str          0 : { *(.debug_str*) }
    .debug_line         0 : { *(.debug_line*) }
    .debug_line_str     0 : { *(.debug_line_str*) }
    .debug_loc          0 : { *(.debug_loc*) }
    .debug_loclists     0 : { *(.debug_loclists*) }

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
        *(.dynsym)
        *(.dynstr)
        *(.hash)
        *(.gnu.hash)
        *(.dynamic)
    }
}
