use crate::abort;

extern "C" {
    static mut __sel4_ipc_buffer_obj: sel4::sys::seL4_IPCBuffer;
}

pub(crate) unsafe fn get_ipc_buffer() -> sel4::IPCBuffer {
    sel4::IPCBuffer::from_ptr(&mut __sel4_ipc_buffer_obj)
}

#[no_mangle]
#[used(linker)]
#[link_section = ".data"]
static mut passive: bool = false; // just a placeholder

/// Returns whether this projection domain is a passive server.
pub fn pd_is_passive() -> bool {
    unsafe { passive }
}

#[no_mangle]
#[used(linker)]
#[link_section = ".data"]
static sel4cp_name: [u8; 16] = [0; 16];

/// Returns the name of this projection domain.
pub fn pd_name() -> &'static str {
    // abort to avoid recursive panic
    fn on_err<T, U>(_: T) -> U {
        abort!("invalid embedded protection domain name");
    }
    core::ffi::CStr::from_bytes_until_nul(&sel4cp_name)
        .unwrap_or_else(&on_err)
        .to_str()
        .unwrap_or_else(&on_err)
}

#[macro_export]
macro_rules! var {
    ($(#[$attrs:meta])* $symbol:ident) => {{
        $(#[$attrs])*
        #[no_mangle]
        #[link_section = ".data"]
        static mut $symbol: usize = 0;

        unsafe {
            $symbol
        }
    }};
}

pub use var;
