# Rust support for seL4 userspace

This repository contains crates for supporting the use of Rust in seL4 userspace. So far, this includes:

- Rust bindings to the seL4 API ([source](./crates/sel4))
- A runtime for root tasks ([source](./crates/sel4-root-task-runtime))
- A runtime for seL4 Core Platform protection domains ([source](./crates/sel4cp))
- A loader for the seL4 kernel ([source](./crates/sel4-loader))
- A CapDL loader ([source](./crates/capdl))

The [./hacking](./hacking) directory contains code for building and testing these crates using Nix and, optionally, Docker. However, the crates in this repository are in no way bound to that build system code.

This work is funded by the [seL4 Foundation](https://sel4.systems/Foundation/home.pml).

### Rendered rustdoc

[https://coliasgroup.gitlab.io/rust-seL4-html/](https://coliasgroup.gitlab.io/rust-seL4-html/)

### Overview of crates

##### General crates

- [`sel4`](./crates/sel4): Straightforward, pure-Rust bindings to the seL4 API.
- [`sel4-sys`](./crates/sel4/sys): Raw bindings to the seL4 API, generated from the libsel4 headers and interface definition files. This crate is not intended to be used directly by application code, but rather serve as a basis for the `sel4` crate's implementation.
- [`sel4-config`](./crates/sel4/config): Macros and constants corresponding to the seL4 kernel configuration. Can be used by all targets (i.e. in all of: application code, build scripts, and build-time tools).
- [`sel4-platform-info`](./crates/sel4-platform-info): Constants corresponding to the contents of `platform_info.h`. Can be used by all targets.
- [`sel4-sync`](./crates/sel4-sync): Synchronization constructs using seL4 IPC. Currently only supports notification-based mutexes.
- [`sel4-logging`](./crates/sel4-logging): Log implementation for the [`log`](https://crates.io/crates/log) crate.

##### Context-specific crates

- **Root task**:
  - [`sel4-root-task-runtime`](./crates/sel4-root-task-runtime): A runtime for root tasks which supports thread-local storage and unwinding, and provides a global allocator.
- **seL4 Core Platform**:
  - [`sel4cp`](./crates/sel4cp): A runtime for [seL4 Core Platform](https://github.com/BreakawayConsulting/sel4cp) protection domains, including an implementation of libsel4cp and abstractions for IPC.

##### Programs

- [Kernel loader](./crates/sel4-loader): A loader for the seL4 kernel, similar in purpose to [elfloader](https://github.com/seL4/seL4_tools/tree/master/elfloader-tool).
- [CapDL loader](./crates/capdl): A CapDL loader.

### Integrating these crates into your project

The best way to learn how to integrate these crates into your project is to check out these concrete examples of their use:

- Simple root task: https://gitlab.com/coliasgroup/rust-seL4-demos/simple-build-system-demo
- Using the seL4 Core Platform: https://gitlab.com/coliasgroup/rust-seL4-demos/simple-sel4cp-demo
- Using the CapDL loader: https://gitlab.com/coliasgroup/rust-seL4-demos/simple-capdl-loader-demo

Some of these crates depend, at build time, on external components and configuration.
In all cases, information for locating these dependencies is passed to the dependant crates via environment variables which are interpreted by `build.rs` scripts.
Here is a list of environment variables that the crates which use them:

- `sel4-config` and `sel4-sys` (whose dependants include `sel4`, `sel4cp`, and many more) use
  `$SEL4_INCLUDE_DIRS`, defaulting to `$SEL4_PREFIX/libsel4/include` if `$SEL4_PREFIX` is set, which
  must contain a colon-separated list of include paths for the libsel4 headers. See the rustdoc for
  the `sel4` crate's rustdoc for more information.
- `sel4-platform-info` (whose dependants include `sel4-loader`) uses `$SEL4_PLATFORM_INFO`,
  defaulting to `$SEL4_PREFIX/support/platform_gen.yaml` if `$SEL4_PREFIX` is set, which must
  contain the path of a `platform_gen.yaml` file from the seL4 kernel build system.
- `sel4-loader` uses `$SEL4_KERNEL`, defaulting to `$SEL4_PREFIX/bin/kernel.elf` if `$SEL4_PREFIX`
  is set, which must contain the path of the seL4 kernel (as an ELF executable). Furthermore, if
  `$SEL4_LOADER_CONFIG` is set, then `sel4-loader` overrides the default configuration with one in
  the provided JSON file. Note that no configuration options are actually implemented yet!

### Running the tests in this repository (quick start)

The only requirements for running the tests in this repository are Git, Make, and Docker.

First, clone this repository:

```
git clone https://gitlab.com/coliasgroup/rust-seL4
cd rust-seL4
```

Next, build, run, and enter a Docker container for development:

```
cd hacking/docker && make run && make exec
```

Inside the container at the repository's top-level directory, build and simulate a simple seL4-based system with a root task written in Rust (this will take a few minutes):

```
make example
```

Also inside the container at the repository's top-level directory, build and run all of this repository's automated tests:

```
make run-automated-tests
```
