use anyhow::Result;
use clap::{App, Arg, ArgAction};

#[derive(Debug)]
pub struct Args {
    pub verbose: bool,
    pub in_file_path: String,
    pub out_file_path: String,
    pub symbols_names: SymbolNames,
}

#[derive(Debug)]
pub struct SymbolNames {
    pub num_phdrs: String,
    pub phdrs: String,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let matches = App::new("")
            .arg(Arg::new("verbose").short('v').action(ArgAction::SetTrue))
            .arg(Arg::new("in_file").value_name("IN_FILE").required(true))
            .arg(
                Arg::new("out_file")
                    .short('o')
                    .value_name("OUT_FILE")
                    .required(true),
            )
            .arg(
                Arg::new("num_phdrs_symbol")
                    .long("num-phdr-symbol")
                    .default_value("__num_phdrs")
                    .value_name("NUM_PHDR_SYMBOL"),
            )
            .arg(
                Arg::new("phdrs_symbol")
                    .long("phdr-symbol")
                    .default_value("__phdrs")
                    .value_name("PHDRS_SYMBOL"),
            )
            .get_matches();

        let verbose = *matches.get_one::<bool>("verbose").unwrap();

        let in_file_path = matches.get_one::<String>("in_file").unwrap().to_owned();
        let out_file_path = matches.get_one::<String>("out_file").unwrap().to_owned();

        let symbols_names = SymbolNames {
            num_phdrs: matches
                .get_one::<String>("num_phdrs_symbol")
                .unwrap()
                .to_owned(),
            phdrs: matches
                .get_one::<String>("phdrs_symbol")
                .unwrap()
                .to_owned(),
        };

        Ok(Args {
            verbose,
            in_file_path,
            out_file_path,
            symbols_names,
        })
    }
}
