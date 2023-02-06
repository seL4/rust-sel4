use sel4_runtime_phdrs::elf::ProgramHeader;
use sel4_runtime_phdrs::embedded::get_phdrs;
use sel4_runtime_phdrs::unwinding::{
    set_custom_eh_frame_finder, EhFrameFinder, ProgramHeadersEhFrameFinder, ProgramHeadersFinder,
};

struct InjectedProgramHeaders;

impl ProgramHeadersFinder for InjectedProgramHeaders {
    fn get_phdrs(&self) -> &[ProgramHeader] {
        get_phdrs()
    }
}

static EH_FRAME_FINDER: &(dyn EhFrameFinder + Sync) =
    &ProgramHeadersEhFrameFinder::new(InjectedProgramHeaders);

pub fn init() {
    set_custom_eh_frame_finder(EH_FRAME_FINDER).unwrap();
}
