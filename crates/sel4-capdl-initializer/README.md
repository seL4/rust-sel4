<!--
     Copyright 2023, Colias Group, LLC

     SPDX-License-Identifier: CC-BY-SA-4.0
-->

# CapDL-based system initializer

This document assumes familiarity with the upstream
[`capdl-loader-app`](https://github.com/seL4/capdl/tree/master/capdl-loader-app).

## `sel4-capdl-initializer`

The `sel4-capdl-initializer` crate is simlar in purpose to the upstream
[`capdl-loader-app`](https://github.com/seL4/capdl/tree/master/capdl-loader-app). The main practical
difference is how it fits into downstream build systems, allowing for different approaches to
distribution and integration. `capdl-loader-app` is a C program built using CMake and depends on the
system CapDL spec at link-time. The `sel4-capdl-initializer` crate itself is just the initializer
without the spec, and thus does not depend on the spec at build time. Instead, it is accompanied by
a CLI component called `sel4-capdl-initializer-add-spec` which is used to append the spec to the
`sel4-capdl-initializer` ELF image in a seperate preparation phase.

`sel4-capdl-initializer` depends on the `libsel4` headers. These are provided in the same way as
they are for the `sel4-config` crate and its dependants (i.e. via `SEL4_PREFIX` or
`SEL4_INCLUDE_DIRS`).

Here is an example of how to build and use this crate, from the root directory of this repository.
First, independantly of the application, build the initializer:

```bash
SEL4_PREFIX=$my_sel4_prefix \
    cargo build \
        -Z build-std=core,alloc,compiler_builtins \
        -Z build-std-features=compiler-builtins-mem \
        --target aarch64-sel4-minimal \
        --release \
        -p sel4-capdl-initializer
```

Later, prepare the initializer by adding the CapDL spec. `sel4-capdl-initializer-add-spec` takes a
CapDL spec in JSON format. [This branch](https://github.com/coliasgroup/capdl/tree/rust) of the
parse-capDL tool is capable of translating a `.cdl` file to JSON.

```bash
parse-capDL --object-sizes=$my_object_sizes --json=spec.json $my_capdl_spec

cargo run -p sel4-capdl-initializer-add-spec -- \
    -e target/aarch64-sel4-minimal/release/sel4-capdl-initializer.elf \
    -f spec.json \
    -d $my_fill_dir \
    -o app.elf
```

There are other ways to acquire and build this code. For example, one could use `cargo install`
without having to clone this repository:

```bash
url="https://github.com/seL4/rust-sel4"

RUST_TARGET_PATH=$my_rust_target_path \
SEL4_PREFIX=$my_sel4_prefix \
    cargo install \
        -Z unstable-options \
        -Z build-std=core,alloc,compiler_builtins \
        -Z build-std-features=compiler-builtins-mem \
        --target aarch64-sel4-minimal \
        --release \
        --root $my_project_local_cargo_root \
        --git $url \
        sel4-capdl-initializer

cargo install \
    --root $my_project_local_cargo_root \
    --git $url \
    sel4-capdl-initializer-add-spec

parse-capDL --object-sizes=$my_object_sizes --json=spec.json $my_capdl_spec

$my_project_local_cargo_root/bin/sel4-capdl-initializer-add-spec \
    -e $my_project_local_cargo_root/bin/sel4-capdl-initializer \
    -f spec.json \
    -d $my_fill_dir \
    -o app.efl
```

## `sel4-capdl-initializer-with-embedded-spec`

`sel4-capdl-initializer-with-embedded-spec` shares most of its code with `sel4-capdl-initializer`,
but is built more like the upsream `sel4-capdl-loader-app`. The resulting root-task memory footprint
is smaller than that of `sel4-capdl-initializer`, but building it is more complex. Instead of
building the binary independantly of the spec and then adding the spec later, the spec is provided
to its `build.rs` script at compile time via environment variables.

```bash
SEL4_PREFIX=$my_sel4_prefix \
CAPDL_SPEC_FILE=spec.json \
CAPDL_FILL_DIR=$my_fill_dir \
    cargo build \
        -Z build-std=core,alloc,compiler_builtins \
        -Z build-std-features=compiler-builtins-mem \
        --target aarch64-sel4-minimal \
        --release \
        -p sel4-capdl-initializer-with-embedded-spec
```
