use regex::Regex;
use xmltree::{Element, XMLNode};

use sel4_config_data::Configuration;

#[derive(Debug, Clone)]
pub enum Condition {
    Variable(String),
    Not(Box<Condition>),
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

impl Condition {
    pub fn parse(e: &Element) -> Self {
        Self::parse_unary(e)
    }

    fn parse_unary(e: &Element) -> Self {
        assert_eq!(e.children.len(), 1);
        Self::parse_inner(e.children[0].as_element().unwrap())
    }

    fn parse_n_ary(e: &Element) -> Vec<Self> {
        e.children
            .iter()
            .filter_map(XMLNode::as_element)
            .map(Self::parse_inner)
            .collect()
    }

    fn parse_inner(e: &Element) -> Self {
        match e.name.as_str() {
            "config" => Self::Variable(Self::parse_var(e.attributes.get("var").unwrap())),
            "not" => Self::Not(Box::new(Self::parse_unary(e))),
            "and" => Self::And(Self::parse_n_ary(e)),
            "or" => Self::Or(Self::parse_n_ary(e)),
            _ => panic!(),
        }
    }

    fn parse_var(var: &str) -> String {
        Regex::new(r"^CONFIG_(.+)$")
            .unwrap()
            .captures(&var)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_owned()
    }

    // // //

    pub fn eval(&self) -> bool {
        match self {
            Condition::Variable(name) => Self::defined(name),
            Condition::Not(inner) => !inner.eval(),
            Condition::And(inners) => inners.iter().all(Self::eval),
            Condition::Or(inners) => inners.iter().any(Self::eval),
        }
    }

    pub fn eval_option(condition: &Option<Self>) -> bool {
        match condition {
            Some(condition) => condition.eval(),
            None => true,
        }
    }

    fn defined(name: &str) -> bool {
        // Unlike in the `sel4_cfg` macro, we interpret undefined values as false rather than failing
        Self::get_kernel_config()
            .get(name)
            .map(|value| value.as_bool().unwrap())
            .unwrap_or(false)
    }

    fn get_kernel_config() -> &'static Configuration {
        sel4_config_data::get_kernel_config()
    }
}
