//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::panic::UnwindSafe;

use sel4_microkit_base::ipc_buffer_ptr;
use sel4_panicking::catch_unwind;
use sel4_panicking_env::abort;

use crate::{panicking::init_panicking, Handler};

#[cfg(target_thread_local)]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    unsafe extern "C" fn cont_fn(_cont_arg: *mut sel4_runtime_common::ContArg) -> ! {
        inner_entry()
    }

    sel4_runtime_common::initialize_tls_on_stack_and_continue(cont_fn, core::ptr::null_mut())
}

#[cfg(not(target_thread_local))]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    inner_entry()
}

#[allow(unreachable_code)]
fn inner_entry() -> ! {
    #[cfg(all(feature = "unwinding", panic = "unwind"))]
    {
        sel4_runtime_common::set_eh_frame_finder().unwrap();
    }

    init_panicking();

    let ipc_buffer = unsafe { ipc_buffer_ptr().as_mut().unwrap() };
    sel4::set_ipc_buffer(ipc_buffer);

    sel4_ctors_dtors::run_ctors();

    unsafe {
        __sel4_microkit__main();
    }

    abort!("__sel4_microkit__main returned")
}

extern "C" {
    fn __sel4_microkit__main() -> !;
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_init {
    ($init:expr) => {
        #[allow(non_snake_case)]
        #[no_mangle]
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
