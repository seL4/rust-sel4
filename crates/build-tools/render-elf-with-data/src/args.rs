use anyhow::{anyhow, ensure, Result};
use clap::{App, Arg, ArgAction};

#[derive(Debug)]
pub struct Args {
    pub in_file_path: String,
    pub out_file_path: String,
    pub injections: Vec<InjectionArg>,
    pub image_bounds_symbols: BoundsSymbolArgs,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct InjectionArg {
    pub file_path: String,
    pub align: Option<usize>,
    pub bounds_symbols: BoundsSymbolArgs,
}

#[derive(Debug, Default)]
pub struct BoundsSymbolArgs {
    pub start: Vec<String>,
    pub end: Vec<String>,
    pub size: Vec<String>,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let matches = App::new("")
            .arg(Arg::new("in_file").value_name("IN_FILE").required(true))
            .arg(
                Arg::new("out_file")
                    .short('o')
                    .value_name("OUT_FILE")
                    .required(true),
            )
            .arg(
                Arg::new("image_bounds_symbols")
                    .long("record-image-bounds")
                    .value_name("[start=SYMBOL][,end=SYMBOL][,size=SYMBOL]"),
            )
            .arg(
                Arg::new("injections")
                    .short('i')
                    .long("inject")
                    .value_name("FILE[,align=ALIGN][,start=SYMBOL][,end=SYMBOL][,size=SYMBOL]")
                    .action(ArgAction::Append),
            )
            .arg(Arg::new("verbose").short('v').action(ArgAction::SetTrue))
            .get_matches();

        let verbose = *matches.get_one::<bool>("verbose").unwrap();

        let in_file_path = matches.get_one::<String>("in_file").unwrap().to_owned();
        let out_file_path = matches.get_one::<String>("out_file").unwrap().to_owned();

        let mut image_bounds_symbols: BoundsSymbolArgs = Default::default();
        if let Some(value) = matches.get_one::<String>("image_bounds_symbols") {
            for parameter in value.split(',') {
                let (k, v) = parameter
                    .split_once('=')
                    .ok_or(anyhow!("malformed parameter '{}'", parameter))?;
                ensure!(image_bounds_symbols.update(k, v)?);
            }
        }

        let mut injections: Vec<InjectionArg> = vec![];
        if let Some(values) = matches.get_many::<String>("injections") {
            for value in values {
                let mut parameters = value.split(',');
                let data_file_path = parameters
                    .next()
                    .ok_or(anyhow!("malformed parameter list '{}'", value))?;
                let mut injection = InjectionArg::new(data_file_path.to_owned());
                for parameter in parameters {
                    let (k, v) = parameter
                        .split_once('=')
                        .ok_or(anyhow!("malformed parameter '{}'", parameter))?;
                    ensure!(injection.update(k, v)?);
                }
                injections.push(injection);
            }
        }

        Ok(Args {
            in_file_path,
            out_file_path,
            image_bounds_symbols,
            injections,
            verbose,
        })
    }
}

impl InjectionArg {
    fn new(file_path: String) -> Self {
        Self {
            file_path,
            align: Default::default(),
            bounds_symbols: Default::default(),
        }
    }

    fn update(&mut self, k: &str, v: &str) -> Result<bool> {
        Ok(match k {
            "align" => {
                self.align = Some(v.parse()?);
                true
            }
            _ => self.bounds_symbols.update(k, v)?,
        })
    }
}

impl BoundsSymbolArgs {
    fn update(&mut self, k: &str, v: &str) -> Result<bool> {
        Ok(match k {
            "start" => {
                self.start.push(v.to_owned());
                true
            }
            "end" => {
                self.end.push(v.to_owned());
                true
            }
            "size" => {
                self.size.push(v.to_owned());
                true
            }
            _ => false,
        })
    }
}
