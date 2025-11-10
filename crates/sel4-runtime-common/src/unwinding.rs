//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_panicking_env::abort;

use unwinding::custom_eh_frame_finder::{
    EhFrameFinder, FrameInfo, FrameInfoKind, set_custom_eh_frame_finder,
};

use sel4_elf_header::{PT_GNU_EH_FRAME, PT_LOAD};

use crate::locate_phdrs;

struct EhFrameFinderImpl;

unsafe impl EhFrameFinder for EhFrameFinderImpl {
    fn find(&self, pc: usize) -> Option<FrameInfo> {
        let phdrs = locate_phdrs();

        let text_base = phdrs.iter().find_map(|phdr| {
            if phdr.p_type == PT_LOAD {
                let vaddr_range = phdr.vaddr_range();
                if vaddr_range.contains(&pc) {
                    return Some(vaddr_range.start);
                }
            }
            None
        })?;

        let eh_frame_hdr = phdrs.iter().find_map(|phdr| {
            if phdr.p_type == PT_GNU_EH_FRAME {
                return Some(phdr.p_vaddr);
            }
            None
        })?;

        Some(FrameInfo {
            text_base: Some(text_base),
            kind: FrameInfoKind::EhFrameHdr(eh_frame_hdr),
        })
    }
}

static EH_FRAME_FINDER: &(dyn EhFrameFinder + Sync) = &EhFrameFinderImpl;

pub(crate) fn init_unwinding() {
    set_custom_eh_frame_finder(EH_FRAME_FINDER)
        .unwrap_or_else(|_| abort!("failed to initialize stack unwinding"));
}
