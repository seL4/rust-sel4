#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::str::FromStr;

use capdl_embed_spec::{Embedding, FillMap, ObjectNamesLevel, SpecForEmbedding};
use capdl_types::{FileContent, Spec};

const CAPDL_SPEC_FILE_ENV: &str = "CAPDL_SPEC_FILE";
const CAPDL_FILL_DIR_ENV: &str = "CAPDL_FILL_DIR";
const CAPDL_OBJECT_NAMES_LEVEL_ENV: &str = "CAPDL_OBJECT_NAMES_LEVEL";
const CAPDL_DEFLATE_FILL_ENV: &str = "CAPDL_DEFLATE_FILL";

pub struct Footprint {
    pub env_vars: Vec<String>,
    pub paths: Vec<PathBuf>,
}

impl Footprint {
    pub fn tell_cargo(&self) {
        for env_var in self.env_vars.iter() {
            println!("cargo:rerun-if-env-changed={env_var}");
        }

        for path in self.paths.iter() {
            println!("cargo:rerun-if-env-changed={}", path.display());
        }
    }
}

pub fn get_embedding<'a>() -> (Embedding<'a>, Footprint) {
    let mut footprint = Footprint {
        env_vars: vec![],
        paths: vec![],
    };

    let spec: SpecForEmbedding = {
        footprint.env_vars.push(CAPDL_SPEC_FILE_ENV.to_owned());
        let json_path = env::var(CAPDL_SPEC_FILE_ENV)
            .unwrap_or_else(|_| panic!("{} must be set", CAPDL_SPEC_FILE_ENV));
        footprint.paths.push(PathBuf::from_str(&json_path).unwrap());
        let orig =
            serde_json::from_reader::<_, Spec<String, FileContent>>(File::open(json_path).unwrap())
                .unwrap();
        orig.traverse_fill_with_context(|length, content| Ok::<_, !>(content.with_length(length)))
            .into_ok()
    };

    let fill_map: FillMap = {
        footprint.env_vars.push(CAPDL_FILL_DIR_ENV.to_owned());
        let fill_dir = env::var(CAPDL_FILL_DIR_ENV)
            .unwrap_or_else(|_| panic!("{} must be set", CAPDL_FILL_DIR_ENV));
        footprint.paths.push(PathBuf::from_str(&fill_dir).unwrap());
        let mut file_ranges = BTreeSet::new();
        spec.traverse_fill(|content| Ok::<_, !>(file_ranges.insert(content.clone())))
            .into_ok();
        let mut files = BTreeSet::new();
        spec.traverse_fill(|content| Ok::<_, !>(files.insert(content.file.clone())))
            .into_ok();
        let mut files = files
            .iter()
            .map(|file| {
                (
                    file.clone(),
                    File::open(PathBuf::from(&fill_dir).join(file)).unwrap(),
                )
            })
            .collect::<BTreeMap<String, File>>();
        file_ranges
            .into_iter()
            .map(|content| {
                let range = content.file_range();
                let file = files.get_mut(&content.file).unwrap();
                file.seek(SeekFrom::Start(range.start.try_into().unwrap()))
                    .unwrap();
                let n = range.end - range.start;
                let mut buf = vec![0; n];
                file.read(&mut buf).unwrap();
                (content.clone(), Cow::Owned(buf))
            })
            .collect::<FillMap>()
    };

    let object_names_level = env::var(CAPDL_OBJECT_NAMES_LEVEL_ENV)
        .map(|val| match val.parse::<usize>().unwrap() {
            0 => ObjectNamesLevel::None,
            1 => ObjectNamesLevel::JustTCBs,
            2 => ObjectNamesLevel::All,
            n => panic!(
                "unexpected value for {}: {}",
                CAPDL_OBJECT_NAMES_LEVEL_ENV, n
            ),
        })
        .unwrap_or(ObjectNamesLevel::JustTCBs);

    footprint
        .env_vars
        .push(CAPDL_OBJECT_NAMES_LEVEL_ENV.to_owned());

    let deflate_fill = env::var(CAPDL_DEFLATE_FILL_ENV)
        .map(|val| match val.parse::<usize>().unwrap() {
            0 => false,
            1 => true,
            n => panic!("unexpected value for {}: {}", CAPDL_DEFLATE_FILL_ENV, n),
        })
        .unwrap_or(false);

    footprint.env_vars.push(CAPDL_DEFLATE_FILL_ENV.to_owned());

    let embedding = Embedding {
        spec: Cow::Owned(spec),
        fill_map: Cow::Owned(fill_map),
        object_names_level,
        deflate_fill,
    };

    (embedding, footprint)
}
