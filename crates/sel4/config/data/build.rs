//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::io::Write;
use std::fs::File;
use std::path::{Path, PathBuf};

use sel4_build_env::find_in_libsel4_include_dirs;
use sel4_config_types::{Configuration, Value};
use tinyjson::JsonValue;

fn main() {
    let config = {
        let kernel_config = from_path(find_in_libsel4_include_dirs("kernel/gen_config.json"));
        let libsel4_config = from_path(find_in_libsel4_include_dirs("sel4/gen_config.json"));
        let mut this = Configuration::empty();
        this.append(kernel_config);
        this.append(libsel4_config);
        this
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("kernel_config.json");
    let mut out_file = File::create(out_path).unwrap();
    let out_json = JsonValue::Object(config.iter().map(|(k, v)| {
        (k.clone(), match v {
            Value::String(v) => JsonValue::String(v.to_string()),
            Value::Bool(v) => JsonValue::Boolean(*v),
        })
    }).collect());
    let out_json_str: String = out_json.format().unwrap();
    write!(out_file, "{}", out_json_str).unwrap();
}

fn from_path(path: impl AsRef<Path>) -> Configuration {
    let json = std::fs::read_to_string(path).unwrap();
    let json: JsonValue = json.parse().unwrap();
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
