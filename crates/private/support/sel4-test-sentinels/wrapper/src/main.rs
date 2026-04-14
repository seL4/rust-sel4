//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::process::Command;

use anyhow::Error;

fn main() -> Result<(), Error> {
    let mut args = env::args_os();
    let _program_name = args.next();
    let child_program = args.next().expect("usage: wrapper <program> [args...]");
    let child_args = args.collect::<Vec<_>>();
    let mut cmd = Command::new(child_program);
    cmd.args(child_args);
    sel4_test_sentinels_wrapper::default_sentinels()
        .wrap(cmd)?
        .success_ok()?;
    println!();
    Ok(())
}
