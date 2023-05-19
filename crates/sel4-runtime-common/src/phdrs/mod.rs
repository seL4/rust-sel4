mod elf;

use elf::{ElfHeader, ProgramHeader};

#[cfg(all(feature = "tls", target_thread_local))]
mod tls;

#[cfg(all(feature = "tls", target_thread_local))]
pub use tls::locate_tls_image;

#[cfg(feature = "unwinding")]
mod unwinding;

#[cfg(feature = "unwinding")]
pub use self::unwinding::set_eh_frame_finder;

pub(crate) fn locate_phdrs() -> &'static [ProgramHeader] {
    extern "C" {
        static __ehdr_start: ElfHeader;
    }
    unsafe {
        assert!(__ehdr_start.check_magic());
        __ehdr_start.locate_phdrs()
    }
}
