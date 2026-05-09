//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::ops::Range;
use std::path::Path;

use num::Integer;
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Object, ObjectSegment, ReadRef};

use sel4_kernel_loader_payload_types::{
    DtbInfo, Payload, PayloadInfo, Region, UserImageInfo, Word,
};

use crate::platform_info::PlatformInfoForBuildSystem;
use crate::utils::{loadable_segments, virt_footprint, with_elf};

const PAGE_SIZE: u64 = 4096;

pub fn serialize_payload<T: FileHeader>(
    kernel_path: impl AsRef<Path>,
    app_path: impl AsRef<Path>,
    dtb_path: impl AsRef<Path>,
    platform_info: &PlatformInfoForBuildSystem,
) -> Payload {
    let mut builder = Builder::new();

    let kernel_entry = with_elf::<T, _, _>(&kernel_path, |elf| {
        builder.add_segments(elf, |phdr| phdr.p_paddr(elf.endian()).into());
        Word(elf.entry())
    });

    let (user_image, user_image_start) = with_elf::<T, _, _>(&app_path, |elf| {
        let coarse_virt_footprint = coarsen_footprint(&virt_footprint(elf), PAGE_SIZE);
        let coarse_footprint_size = coarse_virt_footprint
            .end
            .strict_sub(coarse_virt_footprint.start);
        let ui_p_reg_end = platform_info
            .memory
            .last()
            .unwrap()
            .end
            .prev_multiple_of(&PAGE_SIZE);
        let ui_p_reg_start = ui_p_reg_end.strict_sub(coarse_footprint_size);
        let pv_offset = ui_p_reg_start.wrapping_sub(coarse_virt_footprint.start);

        builder.add_segments(elf, |phdr| {
            let vaddr = phdr.p_vaddr(elf.endian()).into();
            pv_offset.wrapping_add(vaddr)
        });

        let info = UserImageInfo {
            ui_p_reg_start: Word(ui_p_reg_start),
            ui_p_reg_end: Word(ui_p_reg_end),
            pv_offset: Word(truncate_word::<T>(pv_offset)),
            v_entry: Word(elf.entry()),
        };
        (info, ui_p_reg_start)
    });

    let dtb = {
        let data = fs::read(dtb_path).unwrap();
        let size: u64 = data.len().try_into().unwrap();
        let paddr = user_image_start - size.next_multiple_of(PAGE_SIZE);
        builder.add_region(paddr, size, data);
        Some(DtbInfo {
            addr_p: Word(paddr),
            size: Word(size),
        })
    };

    Payload {
        info: PayloadInfo {
            kernel_entry,
            user_image,
            dtb,
        },
        data: builder.regions,
    }
}

//

struct Builder {
    regions: Vec<Region>,
}

impl Builder {
    fn new() -> Self {
        Self { regions: vec![] }
    }

    fn add_segments<'a, T: FileHeader, R: ReadRef<'a>>(
        &mut self,
        elf: &ElfFile<'a, T, R>,
        f: impl Fn(&T::ProgramHeader) -> u64,
    ) {
        for seg in loadable_segments(elf) {
            let paddr = f(seg.elf_program_header());
            self.add_region(paddr, seg.size(), seg.data().unwrap().to_vec());
        }
    }

    fn add_region(&mut self, start: u64, size: u64, data: Vec<u8>) {
        self.regions.push(Region {
            addr: Word(start),
            size: Word(size),
            data,
        });
    }
}

//

fn coarsen_footprint(footprint: &Range<u64>, granularity: u64) -> Range<u64> {
    let start = footprint.start.prev_multiple_of(&granularity);
    let end = footprint.end.next_multiple_of(granularity);
    start..end
}

fn truncate_word<T: FileHeader>(word: u64) -> u64 {
    if T::is_type_64_sized() {
        word
    } else {
        word & 0xffff_ffff
    }
}
