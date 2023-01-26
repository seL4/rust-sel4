use sel4_runtime_building_blocks_elf::ProgramHeader;
use sel4_runtime_building_blocks_embedded_phdrs::get_phdrs;
use sel4_runtime_building_blocks_unwinding_support::{
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
