#![feature(never_type)]
#![feature(unwrap_infallible)]

use capdl_types::*;

const SPEC: &str = include_str!(concat!(env!("OUT_DIR"), "/spec.json"));

const FILL: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/files.rs"));

pub fn get<'a>() -> SpecForBuildSystem<'a, (FillEntryContentFile, FillEntryContentBytes<'static>)> {
    let spec: SpecForBuildSystem<FillEntryContentFile> = serde_json::from_str(SPEC).unwrap();
    spec.traverse_fill_with_context(|entry| {
        Ok::<_, !>((
            entry.content.clone(),
            FillEntryContentBytes {
                bytes: FILL
                    .iter()
                    .find_map(|(name, bytes)| {
                        if name == &entry.content.file {
                            let i = entry.content.file_offset;
                            Some(&bytes[i..i + entry.length])
                        } else {
                            None
                        }
                    })
                    .unwrap(),
            },
        ))
    })
    .into_ok()
}
