use anyhow::Result;
use clap::{App, Arg, ArgAction};

use crate::ObjectNamesLevel;

#[derive(Debug)]
pub struct Args {
    pub loader_elf_path: String,
    pub spec_json_path: String,
    pub fill_dir_path: String,
    pub out_file_path: String,
    pub object_names_level: ObjectNamesLevel,
    pub verbose: bool,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let matches = App::new("")
            .arg(
                Arg::new("loader_elf")
                    .short('e')
                    .value_name("LOADER")
                    .required(true),
            )
            .arg(
                Arg::new("spec_json")
                    .short('f')
                    .value_name("SPEC_FILE")
                    .required(true),
            )
            .arg(
                Arg::new("fill_dir")
                    .short('d')
                    .value_name("FILL_DIR")
                    .required(true),
            )
            .arg(
                Arg::new("out_file")
                    .short('o')
                    .value_name("OUT_FILE")
                    .required(true),
            )
            .arg(
                Arg::new("object_names_level")
                    .long("object-names-level")
                    .short('n')
                    .value_name("OBJECT_NAMES_LEVEL"),
            )
            .arg(Arg::new("verbose").short('v').action(ArgAction::SetTrue))
            .get_matches();

        let loader_elf_path = matches.get_one::<String>("loader_elf").unwrap().to_owned();
        let spec_json_path = matches.get_one::<String>("spec_json").unwrap().to_owned();
        let fill_dir_path = matches.get_one::<String>("fill_dir").unwrap().to_owned();
        let out_file_path = matches.get_one::<String>("out_file").unwrap().to_owned();

        let object_names_level = matches
            .get_one::<usize>("object_names_level")
            .map(|val| match val {
                0 => ObjectNamesLevel::None,
                1 => ObjectNamesLevel::JustTCBs,
                2 => ObjectNamesLevel::All,
                _ => panic!(),
            })
            .unwrap_or(ObjectNamesLevel::JustTCBs);

        let verbose = *matches.get_one::<bool>("verbose").unwrap();

        Ok(Self {
            loader_elf_path,
            spec_json_path,
            fill_dir_path,
            out_file_path,
            object_names_level,
            verbose,
        })
    }
}
