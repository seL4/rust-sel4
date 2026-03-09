//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::path::Path;

use xmltree::Element;

pub mod invocations;
pub mod syscalls;

mod condition;

use condition::Condition;

fn parse_xml(path: impl AsRef<Path>) -> Element {
    let contents = fs::read(path).unwrap();
    Element::parse(&*contents).unwrap()
}
