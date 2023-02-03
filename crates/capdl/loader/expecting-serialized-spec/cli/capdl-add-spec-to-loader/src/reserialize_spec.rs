use std::collections::BTreeMap;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use capdl_types::*;

pub fn reserialize_spec(spec_json: &[u8], fill_dir_path: impl AsRef<Path>) -> Vec<u8> {
    let input_spec: SpecForBuildSystem<FillEntryContentFile> =
        serde_json::from_reader(spec_json).unwrap();

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
    let final_spec: ConcreteSpec<VecContainer, FillEntryContentDeflatedBytesVia, String> =
        input_spec
            .traverse_fill_with_context(|length, entry| {
                let mut uncompressed = vec![0; length];
                open_files
                    .get(&entry.file)
                    .unwrap()
                    .read_exact_at(&mut uncompressed, entry.file_offset.try_into().unwrap())
                    .unwrap();
                let compressed = FillEntryContentDeflatedBytes::pack(&uncompressed);
                let start = fill.len();
                fill.extend(compressed);
                let end = fill.len();
                Ok::<_, !>(FillEntryContentDeflatedBytesVia {
                    deflated_bytes_range: start..end,
                })
            })
            .into_ok();

    let mut blob = postcard::to_allocvec(&final_spec).unwrap();
    blob.extend(fill);
    blob
}
