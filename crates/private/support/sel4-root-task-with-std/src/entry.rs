//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::panic::{catch_unwind, UnwindSafe};

use crate::{abort, Termination};

sel4_runtime_common::declare_entrypoint! {
    (bootinfo: *const sel4::BootInfo) -> ! {
        let bootinfo = unsafe { sel4::BootInfoPtr::new(bootinfo) };

        let ipc_buffer = unsafe { bootinfo.ipc_buffer().as_mut().unwrap() };
        sel4::set_ipc_buffer(ipc_buffer);

        unsafe {
            __sel4_root_task__main(&bootinfo);
        }
    }
}

unsafe extern "Rust" {
    fn __sel4_root_task__main(bootinfo: &sel4::BootInfoPtr) -> !;
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_main {
    ($main:expr) => {
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
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
