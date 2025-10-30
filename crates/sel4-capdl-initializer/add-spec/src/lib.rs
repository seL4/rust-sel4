//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::path::Path;

use rkyv::util::AlignedVec;

use sel4_capdl_initializer_types::InputSpec;

mod render_elf;
mod reserialize_spec;

// HACK hardcoded
const GRANULE_SIZE_BITS: u8 = 12;

type ArchiveAlignedVec = AlignedVec<16>;

pub fn add_spec(
    initializer_without_spec: &[u8],
    spec: &InputSpec,
    fill_dirs: &[impl AsRef<Path>],
    object_names_level: &ObjectNamesLevel,
    embed_frames: bool,
    deflate: bool,
) -> Vec<u8> {
    let (output_spec, embedded_frame_data_list) = reserialize_spec::reserialize_spec(
        spec,
        fill_dirs,
        object_names_level,
        embed_frames,
        deflate,
        GRANULE_SIZE_BITS,
    );

    let spec_data: ArchiveAlignedVec = output_spec.to_bytes().unwrap();

    let embedded_frame_data = embedded_frame_data_list
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let render_elf_args = render_elf::RenderElfArgs {
        spec_data: &spec_data,
        spec_data_alignment: 1 << ArchiveAlignedVec::ALIGNMENT,
        embedded_frame_data: &embedded_frame_data,
        embedded_frame_data_alignment: 1 << GRANULE_SIZE_BITS,
    };

    match object::File::parse(initializer_without_spec).unwrap() {
        object::File::Elf32(initializer_elf) => render_elf_args.call_with(&initializer_elf),
        object::File::Elf64(initializer_elf) => render_elf_args.call_with(&initializer_elf),
        _ => {
            panic!()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectNamesLevel {
    All,
    JustTcbs,
    None,
}
