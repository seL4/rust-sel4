//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use sel4_capdl_initializer_embed_spec::Embedding;
use sel4_capdl_initializer_types::{InputSpec, ObjectNamesLevel};

const CAPDL_SPEC_FILE_ENV: &str = "CAPDL_SPEC_FILE";
const CAPDL_FILL_DIR_ENV: &str = "CAPDL_FILL_DIR";
const CAPDL_OBJECT_NAMES_LEVEL_ENV: &str = "CAPDL_OBJECT_NAMES_LEVEL";
const CAPDL_DEFLATE_FILL_ENV: &str = "CAPDL_DEFLATE_FILL";
const CAPDL_EMBED_FRAMES_ENV: &str = "CAPDL_EMBED_FRAMES";

// HACK hardcoded
const GRANULE_SIZE_BITS: usize = 12;

pub struct Footprint {
    pub env_vars: Vec<String>,
    pub paths: Vec<PathBuf>,
}

impl Footprint {
    pub fn tell_cargo(&self) {
        for env_var in self.env_vars.iter() {
            println!("cargo::rerun-if-env-changed={env_var}");
        }

        for path in self.paths.iter() {
            println!("cargo::rerun-if-env-changed={}", path.display());
        }
    }
}

pub fn get_embedding<'a>() -> (Embedding<'a>, Footprint) {
    let mut footprint = Footprint {
        env_vars: vec![],
        paths: vec![],
    };

    let granule_size_bits = GRANULE_SIZE_BITS;

    let object_names_level = env::var(CAPDL_OBJECT_NAMES_LEVEL_ENV)
        .map(|val| match val.parse::<usize>().unwrap() {
            0 => ObjectNamesLevel::None,
            1 => ObjectNamesLevel::JustTcbs,
            2 => ObjectNamesLevel::All,
            n => panic!("unexpected value for {CAPDL_OBJECT_NAMES_LEVEL_ENV}: {n}"),
        })
        .unwrap_or(ObjectNamesLevel::JustTcbs);

    footprint
        .env_vars
        .push(CAPDL_OBJECT_NAMES_LEVEL_ENV.to_owned());

    let deflate_fill = env::var(CAPDL_DEFLATE_FILL_ENV)
        .map(|val| match val.parse::<usize>().unwrap() {
            0 => false,
            1 => true,
            n => panic!("unexpected value for {CAPDL_DEFLATE_FILL_ENV}: {n}"),
        })
        .unwrap_or(false);

    footprint.env_vars.push(CAPDL_DEFLATE_FILL_ENV.to_owned());

    let embed_frames = env::var(CAPDL_EMBED_FRAMES_ENV)
        .map(|val| match val.parse::<usize>().unwrap() {
            0 => false,
            1 => true,
            n => panic!("unexpected value for {CAPDL_EMBED_FRAMES_ENV}: {n}"),
        })
        .unwrap_or(false);

    footprint.env_vars.push(CAPDL_EMBED_FRAMES_ENV.to_owned());

    let spec = {
        footprint.env_vars.push(CAPDL_SPEC_FILE_ENV.to_owned());
        let json_path = env::var(CAPDL_SPEC_FILE_ENV)
            .unwrap_or_else(|_| panic!("{CAPDL_SPEC_FILE_ENV} must be set"));
        footprint.paths.push(PathBuf::from_str(&json_path).unwrap());
        InputSpec::parse(&fs::read_to_string(json_path).unwrap())
    };

    let fill_map = {
        footprint.env_vars.push(CAPDL_FILL_DIR_ENV.to_owned());
        let fill_dir = env::var(CAPDL_FILL_DIR_ENV)
            .unwrap_or_else(|_| panic!("{CAPDL_FILL_DIR_ENV} must be set"));
        footprint.paths.push(PathBuf::from_str(&fill_dir).unwrap());
        spec.collect_fill([fill_dir])
    };

    let spec = spec.split_embedded_frames(embed_frames, granule_size_bits);

    let embedding = Embedding {
        spec: Cow::Owned(spec),
        fill_map: Cow::Owned(fill_map),
        object_names_level,
        deflate_fill,
        granule_size_bits,
    };

    (embedding, footprint)
}
