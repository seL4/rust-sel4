//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use roxmltree::Node;

use crate::xml::{Condition, child_elements};

#[derive(Debug)]
pub struct Api {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub structs: Vec<Struct>,
    pub interfaces: Vec<Interface>,
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug)]
pub struct Interface {
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub id: String,
    pub condition: Option<Condition>,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: String,
    pub direction: ParameterDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterDirection {
    In,
    Out,
}

impl Method {
    pub fn partition_parameters(&self) -> (Vec<Parameter>, Vec<Parameter>) {
        self.parameters
            .iter()
            .cloned()
            .partition(|parameter| parameter.direction == ParameterDirection::In)
    }
}

// // //

impl Api {
    pub fn parse(e: Node) -> Self {
        let mut structs = vec![];
        let mut interfaces = vec![];
        for child in child_elements(e) {
            match child.tag_name().name() {
                "struct" => {
                    structs.push(Struct::parse(child));
                }
                "interface" => {
                    interfaces.push(Interface::parse(child));
                }
                _ => {}
            }
        }
        Self {
            name: e.attribute("name").map(ToOwned::to_owned),
            structs,
            interfaces,
        }
    }
}

impl Struct {
    fn parse(e: Node) -> Self {
        let mut members = vec![];
        for child in child_elements(e) {
            match child.tag_name().name() {
                "member" => {
                    members.push(child.attribute("name").unwrap().to_owned());
                }
                _ => {
                    panic!();
                }
            }
        }
        Self {
            name: e.attribute("name").unwrap().to_owned(),
            members,
        }
    }
}

impl Interface {
    fn parse(e: Node) -> Self {
        let mut methods = vec![];
        for child in child_elements(e) {
            if child.tag_name().name() == "method" {
                methods.push(Method::parse(child));
            }
        }
        Self {
            name: e.attribute("name").unwrap().to_owned(),
            methods,
        }
    }
}

impl Method {
    fn parse(e: Node) -> Self {
        let mut condition = None;
        let mut parameters = vec![];
        for child in child_elements(e) {
            match child.tag_name().name() {
                "condition" => {
                    assert!(condition.is_none());
                    condition = Some(Condition::parse(child));
                }
                "param" => {
                    parameters.push(Parameter::parse(child));
                }
                _ => {}
            }
        }
        Self {
            name: e.attribute("name").unwrap().to_owned(),
            id: e.attribute("id").unwrap().to_owned(),
            condition,
            parameters,
        }
    }
}

impl Parameter {
    fn parse(e: Node) -> Self {
        Self {
            name: e.attribute("name").unwrap().to_owned(),
            ty: e.attribute("type").unwrap().to_owned(),
            direction: ParameterDirection::parse(e.attribute("dir").unwrap()),
        }
    }
}

impl ParameterDirection {
    fn parse(v: &str) -> Self {
        match v {
            "in" => Self::In,
            "out" => Self::Out,
            _ => panic!(),
        }
    }
}
