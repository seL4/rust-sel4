use xmltree::{Element, XMLNode};

use crate::xml::Condition;

#[derive(Debug)]
pub struct Syscalls {
    pub api_master: ConfigBlocks,
    pub api_mcs: ConfigBlocks,
    pub debug: ConfigBlocks,
}

pub type ConfigBlocks = Vec<ConfigBlock>;

#[derive(Debug)]
pub struct ConfigBlock {
    pub condition: Option<Condition>,
    pub syscalls: Vec<String>,
}

impl Syscalls {
    pub fn parse(e: &Element) -> Self {
        let mut api_master = None;
        let mut api_mcs = None;
        let mut debug = None;
        for child in e.children.iter().filter_map(XMLNode::as_element) {
            let config_blocks = Some(ConfigBlock::parse_many(child));
            match child.name.as_str() {
                "api-master" => api_master = config_blocks,
                "api-mcs" => api_mcs = config_blocks,
                "debug" => debug = config_blocks,
                _ => panic!(),
            }
        }
        Self {
            api_master: api_master.unwrap(),
            api_mcs: api_mcs.unwrap(),
            debug: debug.unwrap(),
        }
    }
}

impl ConfigBlock {
    pub fn parse(e: &Element) -> Self {
        let mut condition = None;
        let mut syscalls = vec![];
        for child in e.children.iter().filter_map(XMLNode::as_element) {
            match child.name.as_str() {
                "condition" => {
                    assert!(condition.is_none());
                    condition = Some(Condition::parse(child));
                }
                "syscall" => {
                    syscalls.push(child.attributes.get("name").unwrap().to_owned());
                }
                _ => {
                    panic!();
                }
            }
        }
        Self {
            condition,
            syscalls,
        }
    }

    fn parse_many(e: &Element) -> Vec<Self> {
        e.children
            .iter()
            .filter_map(XMLNode::as_element)
            .map(Self::parse)
            .collect()
    }
}
