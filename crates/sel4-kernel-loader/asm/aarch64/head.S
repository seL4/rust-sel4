#include "macros.h"
#include "registers.h"
#include "mm.h"

.global _start;
.global secondary_entry;

.extern main
.extern __primary_stack
.extern __primary_stack_size
.extern secondary_main

.extern clean_and_invalideate_dcache
.extern invalidate_dcache
.extern invalidate_icache

.section ".text.startup"

_start:
    mrs     x0, mpidr_el1
    and     x0, x0, #0xf        // Check processor id
    cbnz    x0, hang            // Hang for all non-primary CPU

    ldr     x0, =__bss_start    // [TODO] GNU LD has __bss_start__ and __bss_end__ which feel more robust
    ldr     x1, =_end

clear_bss_loop:
    str     xzr, [x0], #8
    cmp     x0, x1
    b.lt    clear_bss_loop

    ldr     x9, =__primary_stack_bottom
    ldr     x9, [x9]
    mov     sp, x9
    bl      init_core_state
    b       main

secondary_entry:
    bl      init_core_state
    b       secondary_main

hang:
    wfe
    b       hang


BEGIN_LOCAL_FUNC(init_core_state)
    stp     x29, x30, [sp, #-16]!
    mov     x29, sp

    bl      clean_and_invalideate_dcache

    disable_mmu sctlr_el2, x8

    bl      invalidate_icache

    /*
     *   DEVICE_nGnRnE      000     00000000
     *   DEVICE_nGnRE       001     00000100
     *   DEVICE_GRE         010     00001100
     *   NORMAL_NC          011     01000100
     *   NORMAL             100     11111111
     *   NORMAL_WT          101     10101010
     */
    ldr     x5, =MAIR(0x00, MT_DEVICE_nGnRnE) | \
                 MAIR(0x04, MT_DEVICE_nGnRE) | \
                 MAIR(0x0c, MT_DEVICE_GRE) | \
                 MAIR(0x44, MT_NORMAL_NC) | \
                 MAIR(0xff, MT_NORMAL) | \
                 MAIR(0xaa, MT_NORMAL_WT)
    msr     mair_el2, x5
    ldr     x8, =TCR_T0SZ(48) | TCR_IRGN0_WBWC | TCR_ORGN0_WBWC | TCR_SH0_ISH | TCR_TG0_4K | TCR_PS | TCR_EL2_RES1
    msr     tcr_el2, x8
    isb

    adrp    x8, loader_level_0_table
    msr     ttbr0_el2, x8
    isb

    tlbi    alle2is
    dsb     ish
    isb

    enable_mmu  sctlr_el2, x8

    adrp    x8, arm_vector_table
    msr     vbar_el2, x8

    ic      ialluis
    dsb     ish
    isb

    tlbi    alle2is
    dsb     ish
    isb

    ldp     x29, x30, [sp], #16
    ret

END_FUNC(init_core_state)
