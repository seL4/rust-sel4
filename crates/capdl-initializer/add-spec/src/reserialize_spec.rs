use std::ops::Range;
use std::path::Path;

use capdl_initializer_types::*;

pub fn reserialize_spec<'a>(
    input_spec: &InputSpec,
    fill_dir_path: impl AsRef<Path>,
    object_names_level: &ObjectNamesLevel,
    embed_frames: bool,
    granule_size_bits: usize,
    verbose: bool,
) -> (SpecWithIndirection<'a>, Vec<u8>) {
    let granule_size = 1 << granule_size_bits;

    let fill_map = input_spec.collect_fill(&[fill_dir_path]);

    let mut sources = SourcesBuilder::new();
    let mut num_embedded_frames = 0;
    let final_spec: SpecWithIndirection<'a> = input_spec
        .traverse_names_with_context::<_, !>(|named_obj| {
            Ok(object_names_level
                .apply(named_obj)
                .map(|s| IndirectObjectName {
                    range: sources.append(s.as_bytes()),
                }))
        })
        .into_ok()
        .split_embedded_frames(embed_frames, granule_size_bits)
        .traverse_data::<IndirectDeflatedBytesContent, !>(|key| {
            let compressed = DeflatedBytesContent::pack(fill_map.get(key));
            Ok(IndirectDeflatedBytesContent {
                deflated_bytes_range: sources.append(&compressed),
            })
        })
        .into_ok()
        .traverse_embedded_frames::<IndirectEmbeddedFrame, !>(|fill| {
            num_embedded_frames += 1;
            sources.align_to(granule_size);
            let range = sources.append(&fill_map.get_frame(granule_size, fill));
            Ok(IndirectEmbeddedFrame::new(range.start))
        })
        .into_ok();

    if verbose {
        eprintln!("embedded frames count: {}", num_embedded_frames);
    }

    let mut blob = postcard::to_allocvec(&final_spec).unwrap();
    blob.extend(sources.build());
    (final_spec, blob)
}

struct SourcesBuilder {
    buf: Vec<u8>,
}

impl SourcesBuilder {
    fn new() -> Self {
        Self { buf: vec![] }
    }

    fn build(self) -> Vec<u8> {
        self.buf
    }

    fn align_to(&mut self, align: usize) {
        assert!(align.is_power_of_two());
        self.buf.resize(self.buf.len().next_multiple_of(align), 0);
    }

    fn append(&mut self, bytes: &[u8]) -> Range<usize> {
        let start = self.buf.len();
        self.buf.extend(bytes);
        let end = self.buf.len();
        start..end
    }
}
