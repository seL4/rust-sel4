/*
 * Copyright 2023, Colias Group, LLC
 * Copyright 2014, General Dynamics C4 Systems
 *
 * SPDX-License-Identifier: GPL-2.0-only
 */

#include "macros.h"
#include "registers.h"
#include "mm.h"

.text

BEGIN_FUNC(switch_translation_tables_el2)

    stp     x29, x30, [sp, #-16]!
    mov     x29, sp

    bl      clean_and_invalideate_dcache

    disable_mmu sctlr_el2, x9

    bl      invalidate_icache

    ldr     x9, =kernel_boot_level_0_table
    ldr     x9, [x9]
    msr     ttbr0_el2, x9
    isb

    tlbi    alle2is
    dsb     ish
    isb

    enable_mmu  sctlr_el2, x9

    ic      ialluis
    dsb     ish
    isb

    tlbi    alle2is
    dsb     ish
    isb

    ldp     x29, x30, [sp], #16

    ret

END_FUNC(switch_translation_tables_el2)
