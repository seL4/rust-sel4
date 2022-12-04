use core::ffi::c_int;

use sel4_config::sel4_cfg_if;

use crate::seL4_Word;

sel4_cfg_if! {
    if #[cfg(ARCH_AARCH64)] {
        #[path = "aarch64.rs"]
        mod imp;
    } else if #[cfg(ARCH_RISCV64)] {
        #[path = "riscv64.rs"]
        mod imp;
    } else if #[cfg(ARCH_X86_64)] {
        #[path = "x86_64.rs"]
        mod imp;
    }
}

pub use imp::*;

fn sys_id_to_word(sys_id: c_int) -> seL4_Word {
    sys_id as seL4_Word
}
