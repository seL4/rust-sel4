/*
 * Copyright 2024, Colias Group, LLC
 * Copyright 2014, General Dynamics C4 Systems
 *
 * SPDX-License-Identifier: GPL-2.0-only
 */

#include "macros.h"
#include "registers.h"
#include "mm.h"

.text

BEGIN_FUNC(switch_translation_tables)

    stmfd   sp!, {lr}

    /* Clean D-Cache if enabled */
    mrc     SCTLR(r1)
    and     r1, r1, #(1 << 2)
    cmp     r1, #0
    beq     1f
    bl      flush_dcache
1:
    /* Ensure I-cache, D-cache and mmu are disabled. */
    mrc     SCTLR(r1)
    bic     r1, r1, #(1 << 12)      /* Disable I-cache */
    bic     r1, r1, #(1 << 2)       /* Disable D-Cache */
    bic     r1, r1, #(1 << 0)       /* Disable MMU     */
    mcr     SCTLR(r1)

    /* invalidate caches. */
    bl      invalidate_dcache
    bl      invalidate_icache

    /* Set up TTBR0, enable caching of pagetables. */
    ldr     r0, =kernel_boot_level_0_table
    ldr     r0, [r0]
    orr     r1, r0, #0x19
    // orr     r1, r0, #0
    mcr     TTBR0(r1)
    mcr     TLBIALL(r1)

    /* Setup client to only have access to domain 0, and setup the DACR. */
    mov     r1, #1
    mcr     DACR(r1)

    /* Setup misc MMU. */
    mov     r1, #0
    mcr     CONTEXTIDR(r1)  /* set ASID to 0    */
    mcr     TTBCR(r1)       /* set TTBCR to 0   */
    mcr     BPIALL(r1)      /* flush branch target cache */
    isb

    /* Enable MMU, D-cache, and I-cache. */
    mrc     SCTLR(r0)
    orr     r0, r0, #(1 << 13)      /* selects the base address of the exception vectors */
    orr     r0, r0, #(1 << 12)      /* Enable I-cache */
    orr     r0, r0, #(1 << 2)       /* Enable D-cache */
    orr     r0, r0, #(1 << 0)       /* Enable MMU */
    mcr     SCTLR(r0)

    /* Enable/disable Async aborts to drain pending bootloader aborts */
    cpsie   a
    dsb
    isb
    cpsid   a

    ldmfd   sp!, {pc}

END_FUNC(switch_translation_tables)
