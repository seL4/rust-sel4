<!--
     Copyright 2023, Colias Group, LLC

     SPDX-License-Identifier: CC-BY-SA-4.0
-->

# Kernel loader

`sel4-kernel-loader` is simlar in purpose to the upstream
[`elfloader`](https://github.com/seL4/seL4_tools/tree/master/elfloader-tool). The main practical
difference is how it fits into a build system, allowing for different approaches to distribution and
integration. `elfloader` is a C program built using CMake and depends on the kernel+application
payload at link-time. `sel4-kernel-loader` itself is just the loader without the payload, and thus
does not depend on the payload at build time. Instead, it is accompanied by a CLI component called
`sel4-kernel-loader-add-payload` which is used to append the payload to the `sel4-kernel-loader` ELF
image in a seperate preparation phase.

`sel4-kernel-loader` does, however, depend on the seL4 kernel at compile time. The kernel provided
at build time is only used for computing addresses for memory layout. A different kernel can be
provided to the CLI during the preparation phase, as long as that kernel's physical address range's
start address is the same as that of the kernel provided at boot, and that kernel's physical address
range's end address is no greater than that of the kernel provided at boot.

`sel4-kernel-loader` also depends on the `libsel4` headers. These are provided in the same way as
they are for the `sel4-config` crate and its dependants (i.e. via `SEL4_PREFIX` or
`SEL4_INCLUDE_DIRS`).

Future versions of `sel4-kernel-loader` will be configurable with a JSON file provided at compile
time via `SEL4_KERNEL_LOADER_CONFIG`. If not configuration is provided, defaults will be used.

Here is an example of how to build and use this crate. First, independantly of the application,
build the loader and accompanying CLI tool:

```bash
url="https://github.com/seL4/rust-sel4"

CC=aarch64-linux-gnu-gcc \
SEL4_PREFIX=$my_sel4_prefix \
    cargo install \
        -Z unstable-options \
        -Z build-std=core,alloc,compiler_builtins \
        -Z build-std-features=compiler-builtins-mem \
        --target aarch64-unknown-none \
        --root $my_project_local_cargo_root \
        --git $url \
        sel4-kernel-loader

cargo install \
    --root $my_project_local_cargo_root \
    --git $url \
    sel4-kernel-loader-add-payload
```

Later, prepare the loader by adding the kernel+application payload:

```bash
$my_project_local_cargo_root/bin/sel4-kernel-loader-add-payload \
    --sel4-prefix $my_sel4_prefix \
    --loader $my_project_local_cargo_root/bin/sel4-kernel-loader \
    --app $my_app \
    -o image.elf
```
