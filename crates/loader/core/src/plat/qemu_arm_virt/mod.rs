use crate::init_platform_state::reset_cntvoff;

pub(crate) mod debug;
pub(crate) mod smp;

pub(crate) fn init_platform_state_per_core(_core_id: usize) {
    unsafe {
        reset_cntvoff();
    }
}
