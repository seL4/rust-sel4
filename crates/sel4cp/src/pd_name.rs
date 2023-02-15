use core::ffi::CStr;

use crate::abort;

#[no_mangle]
#[link_section = ".data"]
static sel4cp_name: [u8; 16] = [0; 16];

pub fn get_pd_name() -> &'static str {
    // avoid recursive panic
    fn on_err<T, U>(_: T) -> U {
        abort!("invalid embedded protection domain name");
    }
    CStr::from_bytes_until_nul(&sel4cp_name)
        .unwrap_or_else(&on_err)
        .to_str()
        .unwrap_or_else(&on_err)
}
