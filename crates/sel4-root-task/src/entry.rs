//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::panic::UnwindSafe;

use crate::{abort, panicking::catch_unwind, Termination};

#[cfg(target_thread_local)]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry(bootinfo: *const sel4::BootInfo) -> ! {
    unsafe extern "C" fn cont_fn(cont_arg: *mut sel4_runtime_common::ContArg) -> ! {
        inner_entry(cont_arg.cast_const().cast())
    }

    sel4_runtime_common::initialize_tls_on_stack_and_continue(cont_fn, bootinfo.cast_mut().cast())
}

#[cfg(not(target_thread_local))]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry(bootinfo: *const sel4::BootInfo) -> ! {
    inner_entry(bootinfo)
}

#[allow(unreachable_code)]
fn inner_entry(bootinfo: *const sel4::BootInfo) -> ! {
    #[cfg(all(feature = "unwinding", panic = "unwind"))]
    {
        sel4_runtime_common::set_eh_frame_finder().unwrap();
    }

    let bootinfo = unsafe { sel4::BootInfoPtr::new(bootinfo) };

    let ipc_buffer = unsafe { bootinfo.ipc_buffer().as_mut().unwrap() };
    sel4::set_ipc_buffer(ipc_buffer);

    sel4_ctors_dtors::run_ctors();

    unsafe {
        __sel4_root_task__main(&bootinfo);
    }

    abort!("__sel4_root_task__main returned")
}

extern "Rust" {
    fn __sel4_root_task__main(bootinfo: &sel4::BootInfoPtr) -> !;
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_main {
    ($main:expr) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        fn __sel4_root_task__main(bootinfo: &$crate::_private::BootInfoPtr) -> ! {
            $crate::_private::run_main($main, bootinfo);
        }
    };
}

#[doc(hidden)]
#[allow(clippy::missing_safety_doc)]
pub fn run_main<F, T>(f: F, bootinfo: &sel4::BootInfoPtr) -> !
where
    F: FnOnce(&sel4::BootInfoPtr) -> T + UnwindSafe,
    T: Termination,
{
    let result = catch_unwind(move || f(bootinfo).report());
    match result {
        Ok(err) => abort!("main thread terminated with error: {err:?}"),
        Err(_) => abort!("uncaught panic in main thread"),
    }
}
