//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;
use proc_macro2::TokenStream;

use sel4_build_env::{find_in_libsel4_include_dirs, get_libsel4_include_dirs};

mod bf;
mod c;
mod xml;

const OUT_DIR_ENV: &str = "OUT_DIR";

fn main() {
    check_configuration();

    let out_dir = OutDir::new();

    let mut blocklist_for_bindgen = vec![];

    for d in get_libsel4_include_dirs() {
        for f in glob(&format!("{}/**/*.pbf", d.display()))
            .unwrap()
            .map(Result::unwrap)
        {
            let fragment = bf::generate_rust(&mut blocklist_for_bindgen, &f);
            out_dir.write_file(fragment, f.with_extension("rs").file_name().unwrap());
        }
    }

    {
        let fragment =
            xml::syscalls::generate_rust(find_in_libsel4_include_dirs("api/syscall.xml"));
        out_dir.write_file(fragment, "syscall_ids.rs");
    }

    {
        let interface_definition_files = vec![
            // order must be consistent with C libsel4
            find_in_libsel4_include_dirs("interfaces/object-api.xml"),
            find_in_libsel4_include_dirs("interfaces/object-api-sel4-arch.xml"),
            find_in_libsel4_include_dirs("interfaces/object-api-arch.xml"),
        ];

        let (invocation_labels_fragment, invocations_fragment) = xml::invocations::generate_rust(
            &mut blocklist_for_bindgen,
            &interface_definition_files,
        );

        out_dir.write_file(invocation_labels_fragment, "invocation_labels.rs");
        out_dir.write_file(invocations_fragment, "invocations.rs");
    }

    {
        let bindings = c::generate_rust(get_libsel4_include_dirs(), &blocklist_for_bindgen);
        let out_path = out_dir.path.join("bindings.rs");
        bindings.write_to_file(out_path).unwrap();
    }

    println!(
        "cargo::metadata=include={}",
        get_libsel4_include_dirs()
            .map(|p| p.to_str().unwrap().to_owned())
            .collect::<Vec<_>>()
            .join(":")
    );
}

#[allow(clippy::assertions_on_constants)]
fn check_configuration() {
    assert!(!sel4_config::sel4_cfg_bool!(ARCH_IA32));
}

struct OutDir {
    path: PathBuf,
}

impl OutDir {
    fn new() -> Self {
        Self {
            path: Path::new(&env::var(OUT_DIR_ENV).unwrap()).to_owned(),
        }
    }

    fn write_file(&self, toks: TokenStream, filename: impl AsRef<Path>) {
        let formatted = prettyplease::unparse(&syn::parse2(toks).unwrap());
        let out_path = self.path.join(filename);
        fs::write(out_path, formatted).unwrap();
    }
}
