/*
 * Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
 * Copyright 2023, Colias Group, LLC
 *
 * SPDX-License-Identifier: GPL-2.0-only
 */

#include <kernel/gen_config.h>

.extern __global_pointer$
.extern __primary_stack_bottom
.extern hsm_exists
.extern next_logical_core_id
.extern start_core_by_logical_id
.extern secondary_core_sp
.extern arch_main
.extern arch_secondary_main

#define BIT(n) (1 << (n))

/* SBI commands */
#define SBI_HSM_BASE 0x48534DULL
#define SBI_HSM_BASE_HART_START 0
#define SBI_HSM_BASE_HART_STOP 1
#define SBI_EXT_BASE 0x10
#define SBI_EXT_BASE_PROBE_EXT 3

#if __riscv_xlen == 32
    #define lx lw
#else
    #define lx ld
#endif

.section ".text.start"

/* OpenSBI starts us these parameters:
 *   a0: hart id
 *   a1: dtb
 *
 * On RISC-V, only M-Mode can access the CSR mhartid to get the actual hart ID,
 * the SBI running there is responsible for passing this ID up. In S-Mode there
 * is no way to ever query it again, so we have to preserve what we get passed
 * here. This is a RISC-V design decision, more background can be found at
 * https://github.com/riscv/riscv-sbi-doc/issues/25.
 * It seems that OpenSBI starts us at a random hart and keeps all other harts
 * suspended or spinning. However, even on non-SMP configurations there might
 * be an expectation that we are running on CONFIG_FIRST_HART_ID. If the current
 * hart turns out to be a different one, we have to switch harts somehow. The
 * SBI Heart State Management (HSM) extension exists for this, but it might not
 * be implemented. In this case, there is nothing we can do here in the assembly
 * startup code, but C boot code might still have platform specific proprietary
 * ways to switch harts.
 */

.global _start
_start:

.option push
.option norelax
1:auipc gp, %pcrel_hi(__global_pointer$)
  addi  gp, gp, %pcrel_lo(1b)
.option pop

  /* save the parameters passed */
  mv s0, a0 /* preserve a0 (hart id) in s0 */
  mv s2, a1 /* preserve a1 (dtb) in s2 */

  /* Attach the stack to sp before calling any C functions */
  la sp, __primary_stack_bottom
  lx sp, (sp)

  /* Check if the Heart State Management (HSM) extension exists, so it can be
   * used to switch harts if we are not running on hart CONFIG_FIRST_HART_ID.
   * The SBI returns SBI_SUCCESS (0) in a0 if the call could be processed or an
   * error code if not. On SBI_SUCCESS the value in a1 is 0 if the extension is
   * not available or an extension-specific non-zero value if it is available.
   */
  li a7, SBI_EXT_BASE
  li a6, SBI_EXT_BASE_PROBE_EXT
  li a0, SBI_HSM_BASE
  ecall /* call SBI to probe for HSM extension */
  mv a2, a0 /* move SBI call generic return code to a2 as we need a0 */
  mv a3, a1 /* move SBI call error return code to a3 as we need a1 */
  mv a0, s0 /* restore a0 to hold hart ID passed by the boot loader */
  mv a1, s2 /* restore a1 to hold dtb address passed by the boot loader */
  bnez a2, _start1 /* goto _start1 if SBI did not return SBI_SUCCESS (0) */
  beqz a3, _start1 /* goto _start1 if HSM extension is missing */

  /* Update global bool variable to tell boot code the HSM extension exists. */
  la t1, hsm_exists
  li t2, 1
  amoadd.w t1, t2, (t1)

  /*  Check if we are on CONFIG_FIRST_HART_ID */
  li s1, CONFIG_FIRST_HART_ID
  beq  a0, s1, _start1 /* goto _start1 if we are on CONFIG_FIRST_HART_ID */

  /* Use HSM extension to start hart CONFIG_FIRST_HART_ID. */
hsm_switch_hart:
  li a7, SBI_HSM_BASE
  li a6, SBI_HSM_BASE_HART_START
  li a0, CONFIG_FIRST_HART_ID /* hart id to start */
  mv a2, s2 /* dtb address to be passed in a1 when new hart starts is 3rd parameter */
  la a1, _start1 /* where to start the hart */
  ecall /* call SBI to start hart FIRST_HART_ID */

  /* Since we are not the designated primary hart, continue the boot process as
   * secondary hart
   */
  mv a0, s0 /* restore a0 to hold hart ID passed by OpenSBI */
  j secondary_harts


_start1: /* a0 must hold current hard ID passed by bootloader */
         /* a1 must hold dtb address passed by bootloader */
.option push
.option norelax
1:auipc gp, %pcrel_hi(__global_pointer$)
  addi  gp, gp, %pcrel_lo(1b)
.option pop

  /* Attach the stack to sp before calling any C functions */
  /* This HART may be a different HART to the one that started at _start
   * If we've switched HARTs then the other HART will get a different stack
   * region in secondary_harts. */
  la sp, __primary_stack_bottom
  lx sp, (sp)
  /* The C code expects the registers to be set up as:
   *   a0 = hart id
   *   a1 = dtb
   */
  la s0, arch_main
  jr s0

.text

.global secondary_harts
secondary_harts:

.option push
.option norelax
1:auipc gp, %pcrel_hi(__global_pointer$)
  addi  gp, gp, %pcrel_lo(1b)
.option pop

  la a1, next_logical_core_id
  li t2, 1
  amoadd.w t0, t2, (a1)
  /* now t0 has the logical core id */
  li t2, CONFIG_MAX_NUM_NODES
  bge t0, t2, hsm_stop_hart

  la a1, start_core_by_logical_id
1:lw t2, (a1)
  bne t0, t2, 1b

  la sp, secondary_core_sp
  lx sp, (sp)
  la s0, arch_secondary_main
  jr s0

/* If we get here then the HSM extension exists and the current
 * HART is not going to be used and needs to be stopped. */
hsm_stop_hart:
  li a7, SBI_HSM_BASE
  li a6, SBI_HSM_BASE_HART_STOP
  ecall /* call SBI to stop current HART */

spin_hart:
  wfi
  j spin_hart
