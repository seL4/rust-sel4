//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_imports)]

use std::sync::LazyLock;
use std::path::Path;

pub use sel4_config_types::{Configuration, Value};
use tinyjson::JsonValue;

pub fn get_kernel_config() -> &'static Configuration {
    &KERNEL_CONFIG
}

static KERNEL_CONFIG: LazyLock<Configuration> =
    LazyLock::new(|| serde_json::from_str(KERNEL_CONFIG_JSON).unwrap());

static KERNEL_CONFIG: LazyLock<Configuration> =
    LazyLock::new(|| from_string(KERNEL_CONFIG_JSON));

const KERNEL_CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/kernel_config.json"));

fn from_path(path: impl AsRef<Path>) -> Configuration {
    let json = std::fs::read_to_string(path).unwrap();
    from_string(&json)
}


fn from_string(string: &str) -> Configuration {
  let json: JsonValue = string.parse().unwrap();
  let JsonValue::Object(ref map) = json else {
      panic!("invalid json: {json:#?}");
  };
  Configuration::new(map.into_iter().map(|(k, v)| {
      (k.clone(), match v {
        JsonValue::String(v) => Value::String(v.to_string()),
        JsonValue::Boolean(v) => Value::Bool(*v),
        _ => panic!("unsupported jsonvalue: key '{k}', value '{v:#?}'"),
      })
  }).collect())
}

