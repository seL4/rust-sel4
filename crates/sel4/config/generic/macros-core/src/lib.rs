#![feature(drain_filter)]

mod attr_macros;
mod cfg_if;
mod common_helpers;
mod eval;
mod expr_macros;

use common_helpers::parse_or_return;
use eval::Evaluator;

pub use attr_macros::*;
pub use cfg_if::cfg_if_impl;
pub use expr_macros::*;
