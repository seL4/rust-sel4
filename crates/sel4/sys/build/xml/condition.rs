//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use roxmltree::Node;

use sel4_config_data::Configuration;

use super::child_elements;

#[derive(Debug, Clone)]
pub enum Condition {
    Variable(String),
    Not(Box<Condition>),
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

impl Condition {
    pub fn parse(e: Node) -> Self {
        Self::parse_unary(e)
    }

    fn parse_unary(e: Node) -> Self {
        let mut children = child_elements(e);
        let child = children.next().unwrap();
        assert!(children.next().is_none());
        Self::parse_inner(child)
    }

    fn parse_n_ary(e: Node) -> Vec<Self> {
        child_elements(e).map(Self::parse_inner).collect()
    }

    fn parse_inner(e: Node) -> Self {
        match e.tag_name().name() {
            "config" => Self::Variable(Self::parse_var(e.attribute("var").unwrap())),
            "not" => Self::Not(Box::new(Self::parse_unary(e))),
            "and" => Self::And(Self::parse_n_ary(e)),
            "or" => Self::Or(Self::parse_n_ary(e)),
            _ => panic!(),
        }
    }

    fn parse_var(var: &str) -> String {
        var.strip_prefix("CONFIG_").unwrap().to_owned()
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
