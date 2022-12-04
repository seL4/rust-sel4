#![feature(exit_status_error)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use which::which;

const RUSTFMT_ENV: &str = "RUSTFMT";

pub struct Rustfmt {
    path: Option<PathBuf>,
}

impl Rustfmt {
    pub fn detect() -> Self {
        let path = env::var(RUSTFMT_ENV)
            .map(PathBuf::from)
            .ok()
            .or_else(|| which("rustfmt").ok());
        Self { path }
    }

    pub fn format(&self, path: impl AsRef<Path>) {
        if let Some(rustfmt_path) = &self.path {
            let output = Command::new(rustfmt_path)
                .arg(path.as_ref())
                .output()
                .unwrap();
            output.status.exit_ok().unwrap_or_else(|err| {
                eprint!("{}", str::from_utf8(&output.stderr).unwrap());
                panic!("{:?}", err)
            });
        }
    }
}
