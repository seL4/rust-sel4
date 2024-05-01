//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use num::{NumCast, One, PrimInt, Zero};

use sel4_render_elf_with_data::{FileHeaderExt, Input, SymbolicInjection, SymbolicValue};

pub(crate) struct RenderElfArgs<'a> {
    pub(crate) orig_elf: &'a [u8],
    pub(crate) data: &'a [u8],
    pub(crate) granule_size_bits: usize,
    pub(crate) heap_size: usize,
}

impl<'a> RenderElfArgs<'a> {
    pub(crate) fn call_with<T: FileHeaderExt<Word: PrimInt, Sword: PrimInt>>(&self) -> Vec<u8> {
        let data_len: T::Word = NumCast::from(self.data.len()).unwrap();
        let heap_size: T::Word = NumCast::from(self.heap_size).unwrap();
        let align_modulus = T::Word::one() << self.granule_size_bits;
        let align_residue = (align_modulus - data_len % align_modulus) % align_modulus;
        let memsz = data_len + heap_size;
        let mut input = Input::<T>::default();
        input.symbolic_injections.push(SymbolicInjection {
            align_modulus,
            align_residue,
            content: self.data,
            memsz,
            patches: vec![
                (
                    "sel4_capdl_initializer_serialized_spec_start".to_owned(),
                    SymbolicValue {
                        addend: T::Sword::zero(),
                    },
                ),
                (
                    "sel4_capdl_initializer_heap_start".to_owned(),
                    SymbolicValue {
                        addend: NumCast::from(data_len).unwrap(),
                    },
                ),
            ],
        });
        input
            .image_start_patches
            .push("sel4_capdl_initializer_image_start".to_owned());
        input
            .image_end_patches
            .push("sel4_capdl_initializer_image_end".to_owned());
        input.concrete_patches.push((
            "sel4_capdl_initializer_serialized_spec_size".to_owned(),
            data_len,
        ));
        input
            .concrete_patches
            .push(("sel4_capdl_initializer_heap_size".to_owned(), heap_size));
        input.render_with_data(self.orig_elf).unwrap()
    }
}
