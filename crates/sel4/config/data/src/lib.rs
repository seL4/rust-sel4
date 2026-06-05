//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_imports)]

use std::sync::LazyLock;
use std::path::Path;

use sel4_build_env::find_in_libsel4_include_dirs_runtime;
pub use sel4_config_types::{Configuration, Value};
use tinyjson::JsonValue;

pub fn get_kernel_config() -> &'static Configuration {
    &KERNEL_CONFIG
}

pub fn config_as_bool(string: &str) -> bool {
  get_kernel_config().get(string).map(|v| v.as_bool().expect("expected bool")).expect("missing config key")
}

pub fn config_as_string(string: &str) -> &str {
  get_kernel_config().get(string).map(|v| v.as_str()).expect("expected string").expect("missing config key")
}

#[cfg(feature = "embedded-config")]
static KERNEL_CONFIG: LazyLock<Configuration> =
    LazyLock::new(|| from_string(KERNEL_CONFIG_JSON));

#[cfg(not(feature = "embedded-config"))]
static KERNEL_CONFIG: LazyLock<Configuration> =
    LazyLock::new(|| {
      let kernel_config = from_path(find_in_libsel4_include_dirs_runtime("kernel/gen_config.json"));
      let libsel4_config = from_path(find_in_libsel4_include_dirs_runtime("sel4/gen_config.json"));
      let mut this = Configuration::empty();
      this.append(kernel_config);
      this.append(libsel4_config);
      this
    });

#[cfg(feature = "embedded-config")]
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

