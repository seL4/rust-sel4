# CapDL-based system initializer

This document assumes familiarity with the upstream
[`capdl-loader-app`](https://docs.sel4.systems/projects/capdl/c-loader-app.html).

## `capdl-initializer`

`capdl-initializer` is simlar in purpose to the upstream
[`capdl-loader-app`](https://docs.sel4.systems/projects/capdl/c-loader-app.html). The main practical
difference is how it fits into a build system, allowing for different approaches to distribution and
integration. `capdl-loader-app` is a C program built using CMake and depends on the system CapDL
spec at link-time. `capdl-initializer` itself is just the initializer without the spec, and thus
does not depend on the spec at build time. Instead, it is accompanied by a CLI component called
`capdl-initializer-add-spec` which is used to append the spec to the `capdl-initializer` ELF image
in a seperate preparation phase.

`capdl-initializer` depends on the `libsel4` headers. These are provided in the same way as
they are for the `sel4-config` crate and its dependants (i.e. via `SEL4_PREFIX` or
`SEL4_INCLUDE_DIRS`).

Here is an example of how to build and use this crate. First, independantly of the application,
build the initializer and accompanying CLI tool:

```bash
url="https://github.com/coliasgroup/rust-sel4"

RUST_TARGET_PATH=$my_rust_target_path \
SEL4_PREFIX=$my_sel4_prefix \
    cargo install \
        -Z unstable-options \
        -Z build-std=core,alloc,compiler_builtins \
        -Z build-std-features=compiler-builtins-mem \
        --target aarch64-sel4-minimal \
        --root $my_project_local_cargo_root \
        --git $url \
        capdl-initializer

cargo install \
    --root $my_project_local_cargo_root \
    --git $url \
    capdl-initializer-add-spec
```

Later, prepare the initializer by adding the CapDL spec. `capdl-initializer-add-spec` takes a CapDL
spec in JSON format. [This branch](https://github.com/coliasgroup/capdl/tree/coliasgroup) of the
parse-capDL tool is capable of translating a `.cdl` file to JSON.

```bash
parse-capDL --object-sizes=$my_object_sizes --json=spec.json $my_capdl_spec

$my_project_local_cargo_root/bin/capdl-initializer-add-spec \
    -e $my_project_local_cargo_root/bin/capdl-initializer \
    -f spec.json \
    -d $my_fill_dir \
    -o app.efl
```

## `capdl-initializer-with-embedded-spec`

`capdl-initializer-with-embedded-spec` shares most of its code with `capdl-initializer`, but is
built more like the upsream `capdl-loader-app`. The resulting root-task memory footprint is smaller
than that of `capdl-initializer`, but building it is more complex. Instead of building the binary
independantly of the spec and then adding the spec later, the spec is provided to its `build.rs`
script at compile time via environment variables.

```bash
url="https://github.com/coliasgroup/rust-sel4"

RUST_TARGET_PATH=$my_rust_target_path \
SEL4_PREFIX=$my_sel4_prefix \
CAPDL_SPEC_FILE=spec.json \
CAPDL_FILL_DIR=$my_fill_dir \
    cargo install \
        -Z unstable-options \
        -Z build-std=core,alloc,compiler_builtins \
        -Z build-std-features=compiler-builtins-mem \
        --target aarch64-sel4-minimal \
        --root $my_project_local_cargo_root \
        --git $url \
        capdl-initializer-with-embedded-spec
```
