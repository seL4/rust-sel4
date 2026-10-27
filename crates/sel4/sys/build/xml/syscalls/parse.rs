//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use roxmltree::Node;

use crate::xml::{Condition, child_elements};

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
    pub fn parse(e: Node) -> Self {
        let mut api_master = None;
        let mut api_mcs = None;
        let mut debug = None;
        for child in child_elements(e) {
            let config_blocks = Some(ConfigBlock::parse_many(child));
            match child.tag_name().name() {
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
    pub fn parse(e: Node) -> Self {
        let mut condition = None;
        let mut syscalls = vec![];
        for child in child_elements(e) {
            match child.tag_name().name() {
                "condition" => {
                    assert!(condition.is_none());
                    condition = Some(Condition::parse(child));
                }
                "syscall" => {
                    syscalls.push(child.attribute("name").unwrap().to_owned());
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

    fn parse_many(e: Node) -> Vec<Self> {
        child_elements(e).map(Self::parse).collect()
    }
}
