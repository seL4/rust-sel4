#![feature(never_type)]
#![feature(unwrap_infallible)]

use loader_payload_at_build_time_types::PayloadAtBuildTime;

const PAYLOAD_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/payload.json"));

const CONTENT_SLICES: &[&[u8]] = include!(concat!(env!("OUT_DIR"), "/content-slices.fragment.rs"));

pub fn get_split() -> (PayloadAtBuildTime<usize>, &'static [&'static [u8]]) {
    let payload = serde_json::from_str::<PayloadAtBuildTime<usize>>(PAYLOAD_JSON).unwrap();
    (payload, CONTENT_SLICES)
}

pub fn get() -> PayloadAtBuildTime<&'static [u8]> {
    let (payload, content_slices) = get_split();
    payload
        .traverse(|i| Result::<_, !>::Ok(content_slices[*i]))
        .into_ok()
}
