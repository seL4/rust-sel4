//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::path::Path;

use roxmltree::{Document, Node};

pub mod invocations;
pub mod syscalls;

mod condition;

use condition::Condition;

fn parse_xml<T>(path: impl AsRef<Path>, f: impl FnOnce(Node) -> T) -> T {
    let contents = fs::read_to_string(path).unwrap();
    f(Document::parse(&contents).unwrap().root_element())
}

fn child_elements<'a, 'input>(e: Node<'a, 'input>) -> impl Iterator<Item = Node<'a, 'input>> {
    e.children().filter(Node::is_element)
}
