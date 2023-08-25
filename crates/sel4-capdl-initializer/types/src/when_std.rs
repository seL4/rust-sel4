use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::ops::Deref;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use crate::{FileContent, FileContentRange, Fill, Spec};

pub type InputSpec = Spec<'static, String, FileContentRange, !>;

impl InputSpec {
    pub fn parse(s: &str) -> Self {
        serde_json::from_str::<Spec<String, FileContent, ()>>(s)
            .unwrap()
            .traverse_embedded_frames::<!, !>(|_| panic!())
            .into_ok()
            .traverse_data_with_context(|length, data| Ok::<_, !>(data.with_length(length)))
            .into_ok()
    }

    pub fn collect_fill(&self, fill_dirs: impl IntoIterator<Item = impl AsRef<Path>>) -> FillMap {
        let mut builder = FillMapBuilder::new(fill_dirs);
        self.traverse_data(|key| builder.add(key)).unwrap();
        builder.build()
    }
}

#[derive(Debug, Clone)]
pub struct FillMap {
    fill_data: BTreeMap<FileContentRange, Vec<u8>>,
}

impl FillMap {
    pub fn get(&self, key: &FileContentRange) -> &[u8] {
        self.fill_data.get(key).map(Deref::deref).unwrap()
    }

    pub fn get_frame(&self, frame_size: usize, fill: &Fill<'_, FileContentRange>) -> Vec<u8> {
        let mut frame = vec![0; frame_size];
        for entry in fill.entries.iter() {
            frame[entry.range.clone()].copy_from_slice(self.get(entry.content.as_data().unwrap()))
        }
        frame
    }
}

pub struct FillMapBuilder {
    file_handles: BTreeMap<String, File>,
    fill_data: BTreeMap<FileContentRange, Vec<u8>>,
    fill_dirs: Vec<PathBuf>,
}

impl FillMapBuilder {
    pub fn new(fill_dirs: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        Self {
            file_handles: BTreeMap::new(),
            fill_data: BTreeMap::new(),
            fill_dirs: fill_dirs
                .into_iter()
                .map(|path| path.as_ref().to_owned())
                .collect(),
        }
    }

    pub fn build(self) -> FillMap {
        FillMap {
            fill_data: self.fill_data,
        }
    }

    pub fn add(&mut self, key: &FileContentRange) -> io::Result<()> {
        if !self.file_handles.contains_key(&key.file) {
            let path = self
                .fill_dirs
                .iter()
                .filter_map(|dir| {
                    let path = dir.join(&key.file);
                    if path.exists() {
                        Some(path)
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or_else(|| panic!("file {:?} not found", key.file));
            self.file_handles
                .insert(key.file.to_owned(), File::open(path)?);
        }
        if !self.fill_data.contains_key(key) {
            let mut buf = vec![0; key.file_length];
            self.file_handles
                .get(&key.file)
                .unwrap()
                .read_exact_at(&mut buf, key.file_offset.try_into().unwrap())?;
            self.fill_data.insert(key.clone(), buf);
        }
        Ok(())
    }
}
