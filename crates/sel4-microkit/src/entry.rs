//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::panic::UnwindSafe;

pub use sel4_panicking::catch_unwind;
pub use sel4_panicking_env::abort;

use crate::env::get_ipc_buffer;
use crate::handler::{run_handler, Handler};
use crate::panicking::init_panicking;

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

fn inner_entry() -> ! {
    #[cfg(all(feature = "unwinding", panic = "unwind"))]
    {
        sel4_runtime_common::set_eh_frame_finder().unwrap();
    }

    init_panicking();

    unsafe {
        sel4::set_ipc_buffer(get_ipc_buffer());
        sel4_runtime_common::run_ctors();
        __sel4_microkit__main();
    }

    abort!("main thread returned")
}

extern "C" {
    fn __sel4_microkit__main();
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_init {
    ($init:expr) => {
        #[no_mangle]
        fn __sel4_microkit__main() {
            $crate::_private::run_main($init);
        }
    };
}

#[allow(clippy::missing_safety_doc)]
pub fn run_main<T: Handler>(init: impl FnOnce() -> T + UnwindSafe) {
    let result = catch_unwind(|| match run_handler(init()) {
        Ok(absurdity) => match absurdity {},
        Err(err) => err,
    });
    match result {
        Ok(err) => abort!("main thread terminated with error: {err}"),
        Err(_) => abort!("main thread panicked"),
    }
}
