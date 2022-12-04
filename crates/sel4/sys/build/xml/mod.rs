use std::fs::File;
use std::path::Path;

use xmltree::Element;

pub mod invocations;
pub mod syscalls;

mod condition;

use condition::Condition;

fn parse_xml(path: impl AsRef<Path>) -> Element {
    Element::parse(File::open(path).unwrap()).unwrap()
}
