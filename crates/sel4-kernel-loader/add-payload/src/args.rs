use anyhow::Result;
use clap::{Arg, ArgAction, Command};

#[derive(Debug)]
pub struct Args {
    pub sel4_config_path: String,
    pub kernel_path: String,
    pub dtb_path: String,
    pub platform_info_path: String,
    pub loader_path: String,
    pub app_path: String,
    pub out_file_path: String,
    pub verbose: bool,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let matches = Command::new("")
            .arg(
                Arg::new("sel4-prefix")
                    .long("sel4-prefix")
                    .value_name("SEL4_PREFIX")
                    .required(false),
            )
            .arg(
                Arg::new("sel4-config")
                    .long("sel4-config")
                    .value_name("SEL4_CONFIG")
                    .required(false),
            )
            .arg(
                Arg::new("kernel")
                    .long("kernel")
                    .value_name("KERNEL")
                    .required(false),
            )
            .arg(
                Arg::new("dtb")
                    .long("dtb")
                    .value_name("DTB")
                    .required(false),
            )
            .arg(
                Arg::new("platform-info")
                    .long("platform-info")
                    .value_name("PLATFORM_INFO")
                    .required(false),
            )
            .arg(
                Arg::new("loader")
                    .long("loader")
                    .value_name("LOADER")
                    .required(true),
            )
            .arg(Arg::new("app").long("app").value_name("APP").required(true))
            .arg(
                Arg::new("out_file")
                    .short('o')
                    .value_name("OUT_FILE")
                    .required(true),
            )
            .arg(Arg::new("verbose").short('v').action(ArgAction::SetTrue))
            .get_matches();

        let sel4_prefix = matches.get_one::<String>("sel4-prefix");

        let sel4_config_path = matches
            .get_one::<String>("sel4-config")
            .map(ToOwned::to_owned)
            .or(sel4_prefix
                .map(|prefix| format!("{prefix}/libsel4/include/kernel/gen_config.json")))
            .unwrap();

        let kernel_path = matches
            .get_one::<String>("kernel")
            .map(ToOwned::to_owned)
            .or(sel4_prefix.map(|prefix| format!("{prefix}/bin/kernel.elf")))
            .unwrap();

        let dtb_path = matches
            .get_one::<String>("dtb")
            .map(ToOwned::to_owned)
            .or(sel4_prefix.map(|prefix| format!("{prefix}/support/kernel.dtb")))
            .unwrap();

        let platform_info_path = matches
            .get_one::<String>("platform-info")
            .map(ToOwned::to_owned)
            .or(sel4_prefix.map(|prefix| format!("{prefix}/support/platform_gen.yaml")))
            .unwrap();

        let loader_path = matches.get_one::<String>("loader").unwrap().to_owned();

        let app_path = matches.get_one::<String>("app").unwrap().to_owned();

        let out_file_path = matches.get_one::<String>("out_file").unwrap().to_owned();

        let verbose = *matches.get_one::<bool>("verbose").unwrap();

        Ok(Self {
            sel4_config_path,
            kernel_path,
            dtb_path,
            platform_info_path,
            loader_path,
            app_path,
            out_file_path,
            verbose,
        })
    }
}
