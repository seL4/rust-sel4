#![feature(never_type)]
#![feature(unwrap_infallible)]

use capdl_types::*;

const SPEC: &str = include_str!(concat!(env!("OUT_DIR"), "/spec.json"));

const FILL: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/files.rs"));

pub fn get<'a>() -> Spec<'a, String, (FileContent, BytesContent<'static>)> {
    let spec: Spec<String, FileContent> = serde_json::from_str(SPEC).unwrap();
    spec.traverse_fill_with_context(|length, content| {
        Ok::<_, !>((
            content.clone(),
            BytesContent {
                bytes: FILL
                    .iter()
                    .find_map(|(name, bytes)| {
                        if name == &content.file {
                            Some(&bytes[content.file_offset..][..length])
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
