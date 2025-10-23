//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;

use cfg_if::cfg_if;

pub type SetThreadPointerFn = unsafe extern "C" fn(thread_pointer: usize);

pub const DEFAULT_SET_THREAD_POINTER_FN: SetThreadPointerFn = default_set_thread_pointer;

unsafe extern "C" fn default_set_thread_pointer(thread_pointer: usize) {
    let val = thread_pointer;

    unsafe {
        cfg_if! {
            if #[cfg(target_arch = "aarch64")] {
                asm!("msr tpidr_el0, {val}", val = in(reg) val);
            } else if #[cfg(target_arch = "arm")] {
                asm!("mcr p15, 0, {val}, c13, c0, 2", val = in(reg) val); // tpidrurw
            } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                asm!("mv tp, {val}", val = in(reg) val);
            } else if #[cfg(target_arch = "x86_64")] {
                asm!("wrfsbase {val}", val = in(reg) val);
            } else {
                compile_error!("unsupported architecture");
            }
        }
    }
}
