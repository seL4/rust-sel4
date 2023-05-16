use unwinding::custom_eh_frame_finder::{
    set_custom_eh_frame_finder, EhFrameFinder, FrameInfo, FrameInfoKind,
    SetCustomEhFrameFinderError,
};

use crate::elf::{PT_GNU_EH_FRAME, PT_LOAD};
use crate::{InnerProgramHeadersFinder, ProgramHeadersFinder};

unsafe impl<T: InnerProgramHeadersFinder> EhFrameFinder for ProgramHeadersFinder<T> {
    fn find(&self, pc: usize) -> Option<FrameInfo> {
        let text_base = self.find_phdrs().iter().find_map(|phdr| {
            if phdr.p_type == PT_LOAD {
                let start = phdr.p_vaddr;
                let end = start + phdr.p_memsz;
                let range = start.try_into().unwrap()..end.try_into().unwrap();
                if range.contains(&pc) {
                    return Some(range.start);
                }
            }
            None
        })?;

        let eh_frame_hdr = self.find_phdrs().iter().find_map(|phdr| {
            if phdr.p_type == PT_GNU_EH_FRAME {
                let eh_frame_hdr = phdr.p_vaddr.try_into().unwrap();
                return Some(eh_frame_hdr);
            }
            None
        })?;

        Some(FrameInfo {
            text_base,
            kind: FrameInfoKind::EhFrameHdr(eh_frame_hdr),
        })
    }
}

#[cfg(feature = "embedded-phdrs")]
pub fn set_custom_eh_frame_finder_using_embedded_phdrs() -> Result<(), SetCustomEhFrameFinderError>
{
    static EH_FRAME_FINDER: &(dyn EhFrameFinder + Sync) = &crate::EmbeddedProgramHeaders::finder();
    set_custom_eh_frame_finder(EH_FRAME_FINDER)
}
