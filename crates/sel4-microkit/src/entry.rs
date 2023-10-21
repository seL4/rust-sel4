//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub use sel4_panicking::catch_unwind;
pub use sel4_panicking_env::{abort, debug_print, debug_println};

use crate::env::get_ipc_buffer;
use crate::handler::{run_handler, Handler};
use crate::panicking::init_panicking;

#[cfg(target_thread_local)]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    use core::ffi::c_void;
    use core::ptr;

    unsafe extern "C" fn cont_fn(_cont_arg: *mut c_void) -> ! {
        inner_entry()
    }

    let cont_arg = ptr::null_mut();

    sel4_runtime_common::locate_tls_image()
        .unwrap()
        .initialize_on_stack_and_continue(cont_fn, cont_arg)
}

#[cfg(not(target_thread_local))]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    inner_entry()
}

unsafe extern "C" fn inner_entry() -> ! {
    #[cfg(feature = "unwinding")]
    {
        sel4_runtime_common::set_eh_frame_finder().unwrap();
    }

    init_panicking();
    sel4::set_ipc_buffer(get_ipc_buffer());
    __sel4_microkit_main();
    abort!("main thread returned")
}

extern "C" {
    fn __sel4_microkit_main();
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_init {
    ($init:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn __sel4_microkit_main() {
            $crate::_private::run_main($init);
        }
    };
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn run_main<T: Handler>(init: impl FnOnce() -> T) {
    match catch_unwind(|| run_handler(init()).into_err()) {
        Ok(err) => abort!("main thread terminated with error: {err}"),
        Err(_) => abort!("main thread panicked"),
    }
}
