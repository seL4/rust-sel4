//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::{
    collections::BTreeMap,
    fs::File,
    os::unix::fs::FileExt as _,
    path::{Path, PathBuf},
};

use sel4_capdl_initializer_types::*;

use super::ObjectNamesLevel;

pub fn reserialize_spec(
    input_spec: &InputSpec,
    fill_dirs: &[impl AsRef<Path>],
    object_names_level: &ObjectNamesLevel,
    embed_frames: bool,
    deflate: bool,
    granule_size_bits: u8,
) -> (SpecForInitializer, Vec<Vec<u8>>) {
    let mut filler = Filler::new(fill_dirs);

    let (mut output_spec, embedded_frames_data) = input_spec.embed_fill(
        granule_size_bits,
        |_| embed_frames,
        |d, buf| {
            filler.read(d, buf);
            deflate
        },
    );

    for named_obj in output_spec.objects.iter_mut() {
        let keep = match object_names_level {
            ObjectNamesLevel::All => true,
            ObjectNamesLevel::JustTcbs => matches!(named_obj.object, Object::Tcb(_)),
            ObjectNamesLevel::None => false,
        };
        if !keep {
            named_obj.name = None;
        }
    }

    (output_spec, embedded_frames_data)
}

struct Filler {
    fill_dirs: Vec<PathBuf>,
    file_handles: BTreeMap<String, File>,
}

impl Filler {
    fn new(fill_dirs: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        Self {
            fill_dirs: fill_dirs
                .into_iter()
                .map(|path| path.as_ref().to_owned())
                .collect(),
            file_handles: BTreeMap::new(),
        }
    }

    fn find_path(&self, file: &str) -> PathBuf {
        self.fill_dirs
            .iter()
            .filter_map(|dir| {
                let path = dir.join(file);
                if path.exists() { Some(path) } else { None }
            })
            .next()
            .unwrap_or_else(|| panic!("file {:?} not found", file))
    }

    fn get_handle(&mut self, file: &str) -> &mut File {
        let path = self.find_path(file);
        self.file_handles
            .entry(file.to_owned())
            .or_insert_with(|| File::open(path).unwrap())
    }

    fn read(&mut self, key: &FillEntryContentFileOffset, buf: &mut [u8]) {
        self.get_handle(&key.file)
            .read_exact_at(buf, key.file_offset)
            .unwrap();
    }
}
