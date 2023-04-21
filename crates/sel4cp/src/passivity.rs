#[no_mangle]
#[link_section = ".data"]
static mut passive: bool = false; // just a placeholder

pub fn is_passive() -> bool {
    unsafe { passive }
}
