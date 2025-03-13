//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config_types::Configuration;

mod attr_macros;
mod cfg_if;
mod common_helpers;
mod condition;
mod expr_macros;

use common_helpers::parse_or_return;
use condition::{Condition, EvalError};

pub struct MacroImpls<'a> {
    config: &'a Configuration,
    synthetic_attr: &'a str,
}

impl<'a> MacroImpls<'a> {
    pub fn new(config: &'a Configuration, synthetic_attr: &'a str) -> Self {
        Self {
            config,
            synthetic_attr,
        }
    }

    pub const fn config(&self) -> &'a Configuration {
        self.config
    }

    const fn synthetic_attr(&self) -> &'a str {
        self.synthetic_attr
    }

    fn eval(&self, cond: &Condition) -> Result<bool, EvalError> {
        cond.eval(self.config())
    }
}
