use xmltree::{Element, XMLNode};

use crate::xml::Condition;

#[derive(Debug)]
pub struct Api {
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
    pub fn parse(e: &Element) -> Self {
        let mut structs = vec![];
        let mut interfaces = vec![];
        for child in e.children.iter().filter_map(XMLNode::as_element) {
            match child.name.as_str() {
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
            name: e.attributes.get("name").map(ToOwned::to_owned),
            structs,
            interfaces,
        }
    }
}

impl Struct {
    fn parse(e: &Element) -> Self {
        let mut members = vec![];
        for child in e.children.iter().filter_map(XMLNode::as_element) {
            match child.name.as_str() {
                "member" => {
                    members.push(child.attributes.get("name").unwrap().to_owned());
                }
                _ => {
                    panic!();
                }
            }
        }
        Self {
            name: e.attributes.get("name").unwrap().to_owned(),
            members,
        }
    }
}

impl Interface {
    fn parse(e: &Element) -> Self {
        let mut methods = vec![];
        for child in e.children.iter().filter_map(XMLNode::as_element) {
            if child.name.as_str() == "method" {
                methods.push(Method::parse(child));
            }
        }
        Self {
            name: e.attributes.get("name").unwrap().to_owned(),
            methods,
        }
    }
}

impl Method {
    fn parse(e: &Element) -> Self {
        let mut condition = None;
        let mut parameters = vec![];
        for child in e.children.iter().filter_map(XMLNode::as_element) {
            match child.name.as_str() {
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
            name: e.attributes.get("name").unwrap().to_owned(),
            id: e.attributes.get("id").unwrap().to_owned(),
            condition,
            parameters,
        }
    }
}

impl Parameter {
    fn parse(e: &Element) -> Self {
        Self {
            name: e.attributes.get("name").unwrap().to_owned(),
            ty: e.attributes.get("type").unwrap().to_owned(),
            direction: ParameterDirection::parse(e.attributes.get("dir").unwrap()),
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
