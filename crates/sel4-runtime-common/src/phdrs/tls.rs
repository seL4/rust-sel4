//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_panicking_env::abort;

#[allow(unused_imports)]
use sel4_initialize_tls_on_stack::{
    ContArg, ContFn, SetThreadPointerFn, TlsImage, DEFAULT_SET_THREAD_POINTER_FN,
};

use crate::phdrs::{elf::PT_TLS, locate_phdrs};

pub unsafe fn initialize_tls_on_stack_and_continue(cont_fn: ContFn, cont_arg: ContArg) -> ! {
    locate_phdrs()
        .iter()
        .find(|phdr| phdr.p_type == PT_TLS)
        .map(|phdr| TlsImage {
            vaddr: phdr.p_vaddr.try_into().unwrap(),
            filesz: phdr.p_filesz.try_into().unwrap(),
            memsz: phdr.p_memsz.try_into().unwrap(),
            align: phdr.p_align.try_into().unwrap(),
        })
        .unwrap_or_else(|| abort!())
        .initialize_on_stack_and_continue(CHOSEN_SET_THREAD_POINTER_FN, cont_fn, cont_arg)
}

sel4::sel4_cfg_if! {
    if #[cfg(all(ARCH_X86_64, SET_TLS_BASE_SELF))] {
        const CHOSEN_SET_THREAD_POINTER_FN: SetThreadPointerFn = set_thread_pointer_via_syscall;

        unsafe extern "C" fn set_thread_pointer_via_syscall(val: usize) {
            sel4::sys::seL4_SetTLSBase(val.try_into().unwrap());
        }
    } else {
        const CHOSEN_SET_THREAD_POINTER_FN: SetThreadPointerFn = DEFAULT_SET_THREAD_POINTER_FN;
    }
}
