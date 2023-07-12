#![feature(never_type)]
#![feature(unwrap_infallible)]

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
        .traverse_names_with_context::<_, !>(|named_obj| {
            Ok(named_obj.name.inner().to_common().map(ToOwned::to_owned))
        })
        .into_ok()
        .traverse_data_with_context::<_, !>(|length, data| {
            let mut buf = vec![0; length];
            data.inner().self_contained_copy_out(&mut buf);
            Ok(buf)
        })
        .into_ok()
        .traverse_embedded_frames::<_, !>(|frame| {
            Ok(unsafe {
                slice::from_raw_parts(frame.inner().ptr(), embedding.granule_size()).to_vec()
            })
        })
        .into_ok();

    let input = &embedding.spec;
    let adapted_input: SpecCommon = input
        .traverse_names_with_context::<_, !>(|named_obj| {
            Ok(embedding
                .object_names_level
                .apply(named_obj)
                .map(Clone::clone))
        })
        .into_ok()
        .traverse_data::<_, !>(|key| Ok(embedding.fill_map.get(key).to_vec()))
        .into_ok()
        .traverse_embedded_frames::<_, !>(|fill| {
            Ok(embedding.fill_map.get_frame(embedding.granule_size(), fill))
        })
        .into_ok();

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
