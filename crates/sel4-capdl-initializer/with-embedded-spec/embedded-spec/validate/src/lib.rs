//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::slice;

use sel4_capdl_initializer_types::*;
use sel4_capdl_initializer_with_embedded_spec_build_env::get_embedding;

type SpecCommon = Spec<'static, Option<String>, Vec<u8>, Vec<u8>>;

pub fn run(tell_cargo: bool) {
    let (embedding, footprint) = get_embedding();

    if tell_cargo {
        footprint.tell_cargo();
    }

    let embedded = &sel4_capdl_initializer_with_embedded_spec_embedded_spec::SPEC;
    let adapted_embedded: SpecCommon = embedded
        .traverse_names_with_context(|named_obj| {
            named_obj.name.inner().to_common().map(ToOwned::to_owned)
        })
        .traverse_data_with_context(|length, data| {
            let mut buf = vec![0; length];
            data.inner().self_contained_copy_out(&mut buf);
            buf
        })
        .traverse_embedded_frames(|frame| unsafe {
            slice::from_raw_parts(frame.inner().ptr(), embedding.granule_size()).to_vec()
        });

    let input = &embedding.spec;
    let adapted_input: SpecCommon = input
        .traverse_names_with_context(|named_obj| {
            embedding
                .object_names_level
                .apply(named_obj)
                .map(Clone::clone)
        })
        .traverse_data(|key| embedding.fill_map.get(key).to_vec())
        .traverse_embedded_frames(|fill| {
            embedding.fill_map.get_frame(embedding.granule_size(), fill)
        });

    if adapted_embedded != adapted_input {
        // NOTE for debugging:
        // std::fs::write("embedded.txt", format!("{:#?}", &embedded)).unwrap();
        // std::fs::write("serialized.txt", format!("{:#?}", &serialized)).unwrap();
        panic!("not equal");
    }
}

trait ObjectNameForComparison {
    fn to_common(&self) -> Option<&str>;
}

impl ObjectNameForComparison for Unnamed {
    fn to_common(&self) -> Option<&str> {
        None
    }
}

impl ObjectNameForComparison for Option<&str> {
    fn to_common(&self) -> Option<&str> {
        *self
    }
}

impl ObjectNameForComparison for &str {
    fn to_common(&self) -> Option<&str> {
        Some(self)
    }
}
