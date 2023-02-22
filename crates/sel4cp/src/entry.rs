use core::fmt;

pub use sel4_panicking::catch_unwind;
pub use sel4_panicking_env::{abort, debug_print, debug_println};
pub use sel4cp_macros::main;

use crate::handler::Handler;
use crate::ipc_buffer::get_ipc_buffer;
use crate::panicking::init_panicking;

#[cfg(target_thread_local)]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    use core::ffi::c_void;
    use core::ptr;
    use sel4_runtime_phdrs::EmbeddedProgramHeaders;

    unsafe extern "C" fn cont_fn(_cont_arg: *mut c_void) -> ! {
        inner_entry()
    }

    let cont_arg = ptr::null_mut();

    EmbeddedProgramHeaders::finder()
        .find_tls_image()
        .reserve_on_stack_and_continue(cont_fn, cont_arg)
}

#[cfg(not(target_thread_local))]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    inner_entry()
}

unsafe extern "C" fn inner_entry() -> ! {
    #[cfg(feature = "unwinding")]
    {
        sel4_runtime_phdrs::unwinding::set_custom_eh_frame_finder_using_embedded_phdrs().unwrap();
    }

    init_panicking();
    sel4::set_ipc_buffer(get_ipc_buffer());
    __sel4cp_main();
    abort!("main thread returned")
}

extern "C" {
    fn __sel4cp_main();
}

#[macro_export]
macro_rules! declare_main {
    ($main:path) => {
        #[no_mangle]
        pub unsafe extern "C" fn __sel4cp_main() {
            $crate::_private::run_main($main);
        }
    };
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn run_main<T>(f: impl FnOnce() -> T)
where
    T: Handler,
    T::Error: fmt::Debug,
{
    match catch_unwind(|| f().run().into_err()) {
        Ok(err) => abort!("main thread terminated with error: {err:?}"),
        Err(_) => abort!("main thread panicked"),
    }
}
