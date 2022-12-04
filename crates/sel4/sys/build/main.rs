#![feature(int_roundings)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;
use proc_macro2::TokenStream;

use sel4_build_env::SEL4_INCLUDE_DIRS;
use sel4_rustfmt_helper::Rustfmt;

mod bf;
mod c;
mod xml;

const OUT_DIR_ENV: &str = "OUT_DIR";

fn main() {
    let libsel4 = Libsel4::new();
    let out_dir = OutDir::new();

    let mut blocklist_for_bindgen = vec![];

    for d in libsel4.include_dirs.iter() {
        for f in glob(&format!("{}/**/*.pbf", d.display()))
            .unwrap()
            .map(Result::unwrap)
        {
            let (native_fragment, wrappers_fragment) =
                bf::generate_rust(&mut blocklist_for_bindgen, &f);
            out_dir.write_file(native_fragment, f.with_extension("rs").file_name().unwrap());
            out_dir.write_file(
                wrappers_fragment,
                f.with_extension("wrappers.rs").file_name().unwrap(),
            );
        }
    }

    {
        let fragment = xml::syscalls::generate_rust(libsel4.find("api/syscall.xml"));
        out_dir.write_file(fragment, "syscall_ids.rs");
    }

    {
        let interface_definition_files = [
            // order must be consistent with C libsel4
            "interfaces/sel4.xml",
            "interfaces/sel4-sel4arch.xml",
            "interfaces/sel4-arch.xml",
        ]
        .into_iter()
        .map(|rel| libsel4.find(rel))
        .collect::<Vec<PathBuf>>();

        let (invocation_labels_fragment, native_fragment, wrappers_fragment) =
            xml::invocations::generate_rust(
                &mut blocklist_for_bindgen,
                &interface_definition_files,
            );
        out_dir.write_file(invocation_labels_fragment, "invocation_labels.rs");
        out_dir.write_file(native_fragment, "invocations.rs");
        out_dir.write_file(wrappers_fragment, "invocations.wrappers.rs");
    }

    {
        let bindings = c::generate_rust(libsel4.include_dirs.iter(), &blocklist_for_bindgen);
        let out_path = out_dir.path.join("bindings.rs");
        bindings.write_to_file(out_path).unwrap();
    }

    for d in libsel4.include_dirs.iter() {
        println!("cargo:rerun-if-changed={}", d.display());
    }
}

struct Libsel4 {
    include_dirs: Vec<PathBuf>,
}

impl Libsel4 {
    fn new() -> Self {
        Self {
            include_dirs: SEL4_INCLUDE_DIRS.get(),
        }
    }

    fn find(&self, relative_path: impl AsRef<Path>) -> PathBuf {
        for d in self.include_dirs.iter() {
            let path = Path::new(d).join(relative_path.as_ref());
            if path.exists() {
                return path;
            }
        }
        panic!()
    }
}

struct OutDir {
    path: PathBuf,
    rustfmt: Rustfmt,
}

impl OutDir {
    fn new() -> Self {
        Self {
            path: Path::new(&env::var(OUT_DIR_ENV).unwrap()).to_owned(),
            rustfmt: Rustfmt::detect(),
        }
    }

    fn write_file(&self, fragment: TokenStream, filename: impl AsRef<Path>) {
        let out_path = self.path.join(filename);
        fs::write(&out_path, format!("{}", fragment)).unwrap();
        self.rustfmt.format(&out_path);
    }
}
