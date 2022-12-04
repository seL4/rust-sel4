/*
 * Copyright 2021, Breakaway Consulting Pty. Ltd.
 * Copyright 2022, Colias Group, LLC.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

#include <sel4/config.h>

#define PSR_F_BIT         0x00000040
#define PSR_I_BIT         0x00000080
#define PSR_A_BIT         0x00000100
#define PSR_D_BIT         0x00000200

#define PSR_MODE_EL0t     0x00000000
#define PSR_MODE_EL1t     0x00000004
#define PSR_MODE_EL1h     0x00000005
#define PSR_MODE_EL2t     0x00000008
#define PSR_MODE_EL2h     0x00000009
#define PSR_MODE_SVC_32   0x00000013

#define SCR_RW_BIT        0x00000400
#define SCR_SMD_BIT       0x00000080
#define SCR_RES_BITS      0x00000030
#define SCR_NS_BIT        0x00000001

#define MT_DEVICE_nGnRnE  0
#define MT_DEVICE_nGnRE   1
#define MT_DEVICE_GRE     2
#define MT_NORMAL_NC      3
#define MT_NORMAL         4
#define MT_NORMAL_WT      5
#define MAIR(_attr, _mt)  ((_attr) << ((_mt) * 8))

#define TCR_T0SZ(x)       ((64 - (x)))
#define TCR_T1SZ(x)       ((64 - (x)) << 16)
#define TCR_TxSZ(x)       (TCR_T0SZ(x) | TCR_T1SZ(x))

#define TCR_IRGN0_WBWC    (1 << 8)
#define TCR_IRGN_NC       ((0 << 8) | (0 << 24))
#define TCR_IRGN_WBWA     ((1 << 8) | (1 << 24))
#define TCR_IRGN_WT       ((2 << 8) | (2 << 24))
#define TCR_IRGN_WBnWA    ((3 << 8) | (3 << 24))
#define TCR_IRGN_MASK     ((3 << 8) | (3 << 24))

#define TCR_ORGN0_WBWC    (1 << 10)
#define TCR_ORGN_NC       ((0 << 10) | (0 << 26))
#define TCR_ORGN_WBWA     ((1 << 10) | (1 << 26))
#define TCR_ORGN_WT       ((2 << 10) | (2 << 26))
#define TCR_ORGN_WBnWA    ((3 << 10) | (3 << 26))
#define TCR_ORGN_MASK     ((3 << 10) | (3 << 26))

#define TCR_SH0_ISH       (3 << 12)
#define TCR_SHARED        ((3 << 12) | (3 << 28))

#define TCR_TG0_4K        (0 << 14)
#define TCR_TG0_64K       (1 << 14)
#define TCR_TG1_4K        (2 << 30)
#define TCR_TG1_64K       (3 << 30)

#define TCR_PS_4G         (0 << 16)
#define TCR_PS_64G        (1 << 16)
#define TCR_PS_1T         (2 << 16)
#define TCR_PS_4T         (3 << 16)
#define TCR_PS_16T        (4 << 16)
#define TCR_PS_256T       (5 << 16)

#define TCR_EL2_RES1      ((1 << 23) | (1 << 31))
#define TCR_ASID16        (1 << 36)

#ifdef CONFIG_ARM_PA_SIZE_BITS_40
#define TCR_PS TCR_PS_1T
#else
#define TCR_PS TCR_PS_16T
#endif

/* Assembler Macros */
.macro dcache op
    dsb     sy
    mrs     x0, clidr_el1
    and     x3, x0, #0x7000000
    lsr     x3, x3, #23

    cbz     x3, finished_\op
    mov     x10, #0

loop1_\op:
    add     x2, x10, x10, lsr #1
    lsr     x1, x0, x2
    and     x1, x1, #7
    cmp     x1, #2
    b.lt    skip_\op

    msr     csselr_el1, x10
    isb

    mrs     x1, ccsidr_el1
    and     x2, x1, #7
    add     x2, x2, #4
    mov     x4, #0x3ff
    and     x4, x4, x1, lsr #3
    clz     w5, w4
    mov     x7, #0x7fff
    and     x7, x7, x1, lsr #13

loop2_\op:
    mov     x9, x4

loop3_\op:
    lsl     x6, x9, x5
    orr     x11, x10, x6
    lsl     x6, x7, x2
    orr     x11, x11, x6
    dc      \op, x11
    subs    x9, x9, #1
    b.ge    loop3_\op
    subs    x7, x7, #1
    b.ge    loop2_\op

skip_\op:
    add     x10, x10, #2
    cmp     x3, x10
    b.gt    loop1_\op

finished_\op:
    mov     x10, #0
    msr     csselr_el1, x10
    dsb     sy
    isb
.endm

/*
 * Disable the MMU.
 *
 * Arguments:
 *   system control register for the appropriate exception level
 *   temporary register
 *
 * This clears bits 0, 2 and 12 in the control register
 * which map to M (MMU disable), C (cache disable) and I (icache disable)
 * bits.
 */
.macro disable_mmu sctlr tmp
    mrs     \tmp, \sctlr
    bic     \tmp, \tmp, #(1 << 0)
    bic     \tmp, \tmp, #(1 << 2)
    bic     \tmp, \tmp, #(1 << 12)
    msr     \sctlr, \tmp
    isb
.endm

.macro disable_id_cache sctlr tmp
    mrs     \tmp, \sctlr
    bic     \tmp, \tmp, #(1 << 2)
    bic     \tmp, \tmp, #(1 << 12)
    msr     \sctlr, \tmp
    isb
.endm

/*
 * Enable the MMU.
 *
 * Arguments:
 *   system control register for the appropriate exception level
 *   temporary register
 *
 * This set bits 0, 2 and 12 in the control register
 * which map to M (MMU enable), C (cache enable) and I (icache enable)
 * bits.
 */
.macro enable_mmu sctlr tmp
    mrs     \tmp, \sctlr
    orr     \tmp, \tmp, #(1 << 0)
    orr     \tmp, \tmp, #(1 << 2)
    orr     \tmp, \tmp, #(1 << 12)
    msr     \sctlr, \tmp
    isb
.endm


/* Standard function decorators. */
#define BEGIN_FUNC(_name) \
    .global _name ; \
    .type _name, %function ; \
_name:

#define END_FUNC(_name) \
    .size _name, .-_name


/* Invalidate the D-cache */
BEGIN_FUNC(invalidate_dcache)
    dcache  isw
    ret
END_FUNC(invalidate_dcache)

/* Flush the D-cache */
BEGIN_FUNC(flush_dcache)
    dcache  cisw
    ret
END_FUNC(flush_dcache)

/* Invalidate the I-cache */
BEGIN_FUNC(invalidate_icache)
    ic      iallu
    dsb     nsh
    isb
    ret
END_FUNC(invalidate_icache)

BEGIN_FUNC(disable_caches_hyp)
    stp     x29, x30, [sp, #-16]!
    mov     x29, sp
    bl      flush_dcache
    disable_id_cache sctlr_el2, x9
    ldp     x29, x30, [sp], #16
    ret
END_FUNC(disable_caches_hyp)

/*
 * Switch from to running in EL1 (assumes correctly running in EL2).
 */
BEGIN_FUNC(switch_to_el1)
    /* push frame pointer and link register to the stack */
    stp     x29, x30, [sp, #-16]!
    mov     x29, sp

    bl      flush_dcache

    /* Disable EL2 MMU (stage1) */
    disable_mmu sctlr_el2, x9

    bl      invalidate_icache

    /* Set execution state for EL1 to AArch64 -- disable virtualization */
    mov     x9, #(1 << 31)
    msr     hcr_el2, x9

    /* Disable traps to EL2 */
    /* FIXME: This enables the 'TZ' bit, which seems to be against the design */
    mov     x9, #0x33ff
    msr     cptr_el2, x9
    msr     hstr_el2, xzr

    /* Since stage 2 addressing is disable, clear the base register */
    msr     vttbr_el2, xzr

    /* Disable EL1 MMU (stage1) */
    disable_mmu sctlr_el1 , x9

    /* Set SPSR for EL2
     *   I => interrupts are masked
     *   F => fast interrupts are masked
     *   A => SError interrupts are masked
     *   D => debug exceptions are masked
     *   MODE_EL1h => return to EL1 on eret (using SP_EL1, not SP_EL0)
     */
    mov     x9, #(PSR_F_BIT | PSR_I_BIT | PSR_A_BIT | PSR_D_BIT | PSR_MODE_EL1h)
    msr     spsr_el2, x9

    /* The same stack is reused */
    ldp     x29, x30, [sp], #16
    mov     x10, sp
    msr     sp_el1, x10

    /* set ELR so that it's possible to perform ERET */
    msr     elr_el2, x30
    eret
END_FUNC(switch_to_el1)


BEGIN_FUNC(el1_mmu_enable)
    stp     x29, x30, [sp, #-16]!
    mov     x29, sp

    bl      flush_dcache

    /* Ensure I-cache, D-cache and mmu are disabled for EL1/Stage1 */
    disable_mmu sctlr_el1, x8

    /*
     * Invalidate the local I-cache so that any instructions fetched
     * speculatively are discarded.
     */
    bl      invalidate_icache

    /*
     *   DEVICE_nGnRnE      000     00000000
     *   DEVICE_nGnRE       001     00000100
     *   DEVICE_GRE         010     00001100
     *   NORMAL_NC          011     01000100
     *   NORMAL             100     11111111
     */
    ldr     x5, =MAIR(0x00, MT_DEVICE_nGnRnE) | \
                 MAIR(0x04, MT_DEVICE_nGnRE) | \
                 MAIR(0x0c, MT_DEVICE_GRE) | \
                 MAIR(0x44, MT_NORMAL_NC) | \
                 MAIR(0xff, MT_NORMAL)
    msr     mair_el1, x5

    ldr     x10, =TCR_TxSZ(48) | TCR_IRGN_WBWA | TCR_ORGN_WBWA | TCR_TG0_4K | TCR_TG1_4K | TCR_ASID16 /*| TCR_SHARED*/
    mrs     x9, ID_AA64MMFR0_EL1
    bfi     x10, x9, #32, #3
    msr     tcr_el1, x10

    /* Setup page tables */
    adrp    x8, boot_lvl0_lower
    msr     ttbr0_el1, x8
    adrp    x8, boot_lvl0_upper
    msr     ttbr1_el1, x8
    isb

    /* invalidate all TLB entries for EL1 */
    tlbi    vmalle1is
    dsb     ish
    isb

    enable_mmu sctlr_el1, x8

    /* set up a vector table so that if the low-level kernel
     * initialization code fails, we have some chance of finding
     * out and printing reasonable diagnostics.
     */
    adrp    x8, arm_vector_table
    msr     vbar_el1, x8

    ldp     x29, x30, [sp], #16
    ret
END_FUNC(el1_mmu_enable)


BEGIN_FUNC(el2_mmu_enable)
    stp     x29, x30, [sp, #-16]!
    mov     x29, sp

    bl      flush_dcache

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

    adrp    x8, boot_lvl0_lower
    msr     ttbr0_el2, x8
    isb

    tlbi    alle2is
    dsb     ish
    isb

    enable_mmu  sctlr_el2, x8

    adrp    x8, arm_vector_table
    msr     vbar_el2, x8

    ic  ialluis
    dsb ish
    isb
    tlbi    alle2is
    dsb     ish
    isb
    ldp     x29, x30, [sp], #16
    ret
END_FUNC(el2_mmu_enable)


BEGIN_FUNC(call_psci)
    smc #0
    ret
END_FUNC(call_psci)


.extern exception_handler
.extern exception_register_state

.macro ventry id
.align 7
    /* push some temp registers on the stack */
    stp     x2, x3, [sp, #-16]
    adrp    x2, exception_register_state
    stp     x0, x1, [x2]
    mov     x0, x2
    ldp     x2, x3, [sp, #-16]
    stp     x2, x3, [x0, #16 * 1]
    stp     x4, x5, [x0, #16 * 2]
    stp     x6, x7, [x0, #16 * 3]
    stp     x8, x9, [x0, #16 * 4]
    stp     x10, x11, [x0, #16 * 5]
    stp     x12, x13, [x0, #16 * 6]
    stp     x14, x15, [x0, #16 * 7]
    stp     x16, x17, [x0, #16 * 8]
    stp     x18, x19, [x0, #16 * 9]
    stp     x20, x21, [x0, #16 * 10]
    stp     x22, x23, [x0, #16 * 11]
    stp     x24, x25, [x0, #16 * 12]
    stp     x26, x27, [x0, #16 * 13]
    stp     x28, x29, [x0, #16 * 14]
    mov     x0, \id
    mrs     x1, ESR_EL1
    mrs     x2, FAR_EL1
    b       exception_handler
.endm

.align 12
BEGIN_FUNC(arm_vector_table)
    ventry  #0
    ventry  #1
    ventry  #2
    ventry  #3
    ventry  #4
    ventry  #5
    ventry  #6
    ventry  #7
    ventry  #8
    ventry  #9
    ventry  #10
    ventry  #11
    ventry  #12
    ventry  #13
    ventry  #14
    ventry  #15
END_FUNC(arm_vector_table)
