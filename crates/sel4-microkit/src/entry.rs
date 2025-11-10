//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::panic::UnwindSafe;

use sel4_microkit_base::ipc_buffer_ptr;
use sel4_panicking::catch_unwind;
use sel4_panicking_env::abort;

use crate::{Handler, panicking::init_panicking};

sel4_runtime_common::declare_entrypoint! {
    entrypoint()
}

fn entrypoint() -> ! {
    init_panicking();

    let ipc_buffer = unsafe { ipc_buffer_ptr().as_mut().unwrap() };
    sel4::set_ipc_buffer(ipc_buffer);

    unsafe {
        __sel4_microkit__main();
    }
}

unsafe extern "C" {
    fn __sel4_microkit__main() -> !;
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_init {
    ($init:expr) => {
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        fn __sel4_microkit__main() -> ! {
            $crate::_private::run_main($init);
        }
    };
}

#[doc(hidden)]
#[allow(clippy::missing_safety_doc)]
pub fn run_main<T: Handler>(init: impl FnOnce() -> T + UnwindSafe) -> ! {
    let result = catch_unwind(|| match init().run() {
        #[allow(unreachable_patterns)]
        Ok(absurdity) => match absurdity {},
        Err(err) => err,
    });
    match result {
        Ok(err) => abort!("main thread terminated with error: {err}"),
        Err(_) => abort!("main thread panicked"),
    }
}
