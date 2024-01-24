/*
 * Copyright 2024, Colias Group, LLC
 * Copyright 2014, General Dynamics C4 Systems
 *
 * SPDX-License-Identifier: GPL-2.0-only
 *
 * See:
 * https://developer.arm.com/documentation/ddi0406/c/System-Level-Architecture/Common-Memory-System-Architecture-Features/Caches-and-branch-predictors/Cache-and-branch-predictor-maintenance-operations?lang=en
 */

#include "macros.h"
#include "registers.h"
#include "mm.h"

.text

BEGIN_FUNC(invalidate_dcache)
    stmfd   sp!, {r4-r11,lr}
    dcache  isw
    ldmfd   sp!, {r4-r11,pc}
END_FUNC(invalidate_dcache)

BEGIN_FUNC(invalidate_icache)
    mcr     IIALL(r1)
    bx      lr
END_FUNC(invalidate_icache)

BEGIN_FUNC(flush_dcache)
    stmfd   sp!, {r4-r11,lr}
    dcache  cisw
    ldmfd   sp!, {r4-r11,pc}
END_FUNC(flush_dcache)

BEGIN_FUNC(arm_disable_dcaches)
    stmfd   sp!, {lr}

    /* Clean D-Cache if enabled */
    mrc     SCTLR(r1)
    and     r1, r1, #(1 << 2)
    cmp     r1, #0
    beq     1f
    bl      flush_dcache
1:
    /* disable D-cache disabled. */
    mrc     SCTLR(r1)
    bic     r1, r1, #(1 << 2)       /* Disable D-Cache */
    mcr     SCTLR(r1)

    /* invalidate dcaches. */
    bl      invalidate_dcache

    ldmfd   sp!, {pc}
END_FUNC(arm_disable_dcaches)
