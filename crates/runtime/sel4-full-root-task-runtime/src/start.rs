use core::ffi::c_void;
use core::fmt;

use sel4_panicking::catch_unwind;
use sel4_runtime_building_blocks_abort::abort;
use sel4_runtime_building_blocks_elf::PT_TLS;
use sel4_runtime_building_blocks_embedded_phdrs::get_phdrs;
use sel4_runtime_building_blocks_reserve_tls_on_stack::TlsImage;
use sel4_runtime_building_blocks_termination::Termination;

use crate::panic_hook;

#[no_mangle]
pub unsafe extern "C" fn __rust_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    let cont_arg = bootinfo.cast::<c_void>().cast_mut();
    let tls_image: TlsImage = get_phdrs()
        .iter()
        .find(|phdr| phdr.p_type == PT_TLS)
        .expect("PT_TLS not found")
        .try_into()
        .unwrap();
    tls_image.reserve_on_stack_and_continue(cont_fn, cont_arg)
}

pub unsafe extern "C" fn cont_fn(cont_arg: *mut c_void) -> ! {
    let bootinfo = cont_arg.cast_const().cast::<sel4::sys::seL4_BootInfo>();

    #[cfg(feature = "unwinding")]
    {
        crate::unwinding::init();
    }

    sel4::set_ipc_buffer(sel4::BootInfo::from_ptr(bootinfo).ipc_buffer());
    sel4_panicking::set_hook(&panic_hook);
    __sel4_for_simple_root_task_main(bootinfo);
    abort()
}

extern "C" {
    fn __sel4_for_simple_root_task_main(bootinfo: *const sel4::sys::seL4_BootInfo);
}

#[macro_export]
macro_rules! declare_main {
    ($main:path) => {
        #[no_mangle]
        pub extern "C" fn __sel4_for_simple_root_task_main(
            bootinfo: *const $crate::_private::start::seL4_BootInfo,
        ) {
            $crate::_private::start::run_main($main, bootinfo);
        }
    };
}

pub fn run_main<T>(f: impl Fn(&sel4::BootInfo) -> T, bootinfo: *const sel4::sys::seL4_BootInfo)
where
    T: Termination,
    T::Error: fmt::Debug,
{
    let _ = catch_unwind(|| {
        let bootinfo = unsafe { sel4::BootInfo::from_ptr(bootinfo) };
        let err = f(&bootinfo).report();
        sel4::debug_println!("Terminated with error: {:?}", err);
    });
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use super::run_main;
    pub use sel4::sys::seL4_BootInfo;
}
