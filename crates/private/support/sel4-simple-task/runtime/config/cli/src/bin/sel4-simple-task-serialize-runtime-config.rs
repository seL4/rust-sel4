//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use sel4_simple_task_runtime_config_types::{RuntimeConfig, RuntimeConfigForPacking};

fn main() -> Result<(), std::io::Error> {
    let config = serde_json::from_reader::<_, RuntimeConfigForPacking<PathBuf>>(io::stdin())?;
    let config = config.traverse(fs::read)?;
    let packed = config.pack();
    let unpacked_for_sanity_check = RuntimeConfigForPacking::unpack(&RuntimeConfig::new(&packed))
        .traverse(|bytes| Ok::<_, !>(bytes.to_vec()))
        .into_ok();
    assert_eq!(config, unpacked_for_sanity_check);
    io::stdout().write_all(&packed)?;
    Ok(())
}
