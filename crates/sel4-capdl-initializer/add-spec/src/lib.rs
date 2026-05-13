//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::path::Path;

use rkyv::util::AlignedVec;

use sel4_capdl_initializer_types::InputSpec;
use sel4_patch_elf::dynamic::Patching;
use sel4_phdrs_constants::{PT_SEL4_CAPDL_FRAME_DATA, PT_SEL4_CAPDL_SPEC};

mod reserialize_spec;

// HACK hardcoded
const GRANULE_SIZE_BITS: u8 = 12;

type ArchiveAlignedVec = AlignedVec;

pub fn add_spec(
    initializer_without_spec: &[u8],
    spec: &InputSpec,
    fill_dirs: &[impl AsRef<Path>],
    object_names_level: &ObjectNamesLevel,
    embed_frames: bool,
    deflate: bool,
    initializer_verbosity: u8,
) -> Vec<u8> {
    let (output_spec, embedded_frame_data_list) = reserialize_spec::reserialize_spec(
        spec,
        fill_dirs,
        object_names_level,
        embed_frames,
        deflate,
        initializer_verbosity,
        GRANULE_SIZE_BITS,
    );

    let spec_data: ArchiveAlignedVec = output_spec.to_bytes().unwrap();

    let embedded_frame_data = embedded_frame_data_list
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let parsed = object::File::parse(initializer_without_spec).unwrap();

    let mut patching = Patching::new(&parsed);

    patching.add_data_segment_with_meta_phdr(
        PT_SEL4_CAPDL_FRAME_DATA,
        1 << GRANULE_SIZE_BITS,
        &embedded_frame_data,
    );

    patching.add_data_segment_with_meta_phdr(
        PT_SEL4_CAPDL_SPEC,
        ArchiveAlignedVec::ALIGNMENT.try_into().unwrap(),
        &spec_data,
    );

    patching.finalize()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectNamesLevel {
    All,
    JustTcbs,
    None,
}
