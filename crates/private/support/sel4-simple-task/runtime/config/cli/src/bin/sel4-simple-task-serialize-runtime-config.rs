//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use sel4_simple_task_runtime_config_types::GenericRuntimeConfig;

fn main() -> Result<(), std::io::Error> {
    let config = serde_json::from_reader::<_, GenericRuntimeConfig<PathBuf>>(io::stdin())?;
    let config = config.traverse(fs::read)?;
    let archive = config.to_bytes().unwrap();
    io::stdout().write_all(&archive)?;
    Ok(())
}
