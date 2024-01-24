/*
 * Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
 *
 * SPDX-License-Identifier: GPL-2.0-only
 */

#include "macros.h"

.macro psci_func op
    stmfd    sp!, {r3-r11, lr}
    dsb
    \op #0
    ldmfd    sp!, {r3-r11, pc}
.endm

.arch_extension sec
.arch_extension virt

.text

BEGIN_FUNC(smc_psci_func)
    psci_func smc
END_FUNC(smc_psci_func)

BEGIN_FUNC(hvc_psci_func)
    psci_func hvc
END_FUNC(hvc_psci_func)
