use sel4_runtime_phdrs::unwinding::{set_custom_eh_frame_finder, EhFrameFinder};
use sel4_runtime_phdrs::EmbeddedProgramHeaders;

static EH_FRAME_FINDER: &(dyn EhFrameFinder + Sync) = &EmbeddedProgramHeaders::finder();

pub fn init() {
    set_custom_eh_frame_finder(EH_FRAME_FINDER).unwrap();
}
