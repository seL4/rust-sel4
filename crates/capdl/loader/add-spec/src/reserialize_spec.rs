use std::collections::BTreeMap;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use capdl_loader_types::SpecForSerialization;
use capdl_types::*;

use crate::ObjectNamesLevel;

pub fn reserialize_spec<'a>(
    input_spec: &Spec<'a, String, FileContent>,
    fill_dir_path: impl AsRef<Path>,
    object_names_level: &ObjectNamesLevel,
) -> (SpecForSerialization<'a>, Vec<u8>) {
    let mut open_files = BTreeMap::new();
    input_spec
        .traverse_fill(|content| {
            if !open_files.contains_key(&content.file) {
                open_files.insert(
                    content.file.to_owned(),
                    File::open(PathBuf::from(fill_dir_path.as_ref()).join(&content.file)).unwrap(),
                );
            }
            Ok::<(), !>(())
        })
        .into_ok();

    let mut fill = vec![];
    let final_spec: SpecForSerialization<'a> = input_spec
        .traverse_names_with_context(|obj, name| {
            let name = match object_names_level {
                ObjectNamesLevel::All => Some(name.clone()),
                ObjectNamesLevel::JustTCBs => match obj {
                    Object::TCB(_) => Some(name.clone()),
                    _ => None,
                },
                ObjectNamesLevel::None => None,
            };
            let indirect_name = name.map(|s| {
                let start = fill.len();
                fill.extend(s.bytes());
                let end = fill.len();
                IndirectObjectName { range: start..end }
            });
            Ok::<_, !>(indirect_name)
        })
        .into_ok()
        .traverse_fill_with_context(|length, entry| {
            let mut uncompressed = vec![0; length];
            open_files
                .get(&entry.file)
                .unwrap()
                .read_exact_at(&mut uncompressed, entry.file_offset.try_into().unwrap())
                .unwrap();
            let compressed = DeflatedBytesContent::pack(&uncompressed);
            let start = fill.len();
            fill.extend(compressed);
            let end = fill.len();
            Ok::<_, !>(IndirectDeflatedBytesContent {
                deflated_bytes_range: start..end,
            })
        })
        .into_ok();

    let mut blob = postcard::to_allocvec(&final_spec).unwrap();
    blob.extend(fill);
    (final_spec, blob)
}
