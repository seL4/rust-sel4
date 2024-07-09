/*
 * Copyright 2024, Colias Group, LLC
 * Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
 *
 * SPDX-License-Identifier: GPL-2.0-only
 */

#include <kernel/gen_config.h>

#include "macros.h"
#include "registers.h"
#include "mm.h"

.global _start;
.global secondary_entry;

.extern __primary_stack
.extern __primary_stack_size
.extern arch_main
.extern arch_secondary_main

.extern loader_level_0_table

.section ".text.startup"

_start:
    mov     r0, r2

    cpsid   ifa                 // Disable Async aborts that might be pending from bootloader

#if CONFIG_MAX_NUM_NODES > 1
    /* Enable SMP */
    mrc     ACTLR(r1)
    orr     r1, r1, #(1 << 6)       /* enable SMP bit   */
#ifdef CONFIG_ARM_CORTEX_A9
    orr     r1, r1, #1              /* enable FW bit    */
#endif
    mcr     ACTLR(r1)
#endif /* CONFIG_MAX_NUM_NODES > 1 */

    ldr     r0, =__bss_start    // [TODO] GNU LD has __bss_start__ and __bss_end__ which feel more robust
    ldr     r1, =_end
    mov     r3, #0

clearzi:
    cmp     r0, r1
    beq     clearzi_exit
    str     r3, [r0]
    add     r0, r0, #4
    b       clearzi

clearzi_exit:

    ldr     r9, =__primary_stack_bottom
    ldr     r9, [r9]
    mov     sp, r9
    bl      leave_hyp
    bl      init_core_state
    b       arch_main
    b       hang


secondary_entry:
    bl      leave_hyp
    bl      init_core_state
    b       arch_secondary_main
    b       hang


hang:
    wfe
    b       hang


BEGIN_LOCAL_FUNC(init_core_state)
    stmfd   sp!, {lr}

    mov     r0, #0
    mcr     IIALL(r0)
    dcache  isw

    bl      arm_enable_mmu

    ldmfd   sp!, {pc}
END_FUNC(init_core_state)


BEGIN_FUNC(leave_hyp)
    .arch_extension virt
    mrs r9, cpsr
    ldr r9, =CPSR_SUPERVISOR
    msr sp_svc, sp
    msr spsr_cxsf, r9
    msr elr_hyp, lr
    eret
END_FUNC(leave_hyp)


BEGIN_FUNC(arm_enable_mmu)
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
    ldr     r0, =loader_level_0_table
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
END_FUNC(arm_enable_mmu)
