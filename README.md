# Rust support for seL4 userspace

This repository contains crates for supporting the use of Rust in seL4 userspace. So far, this
includes:

- Rust bindings for the seL4 API ([source](./crates/sel4))
- A runtime for root tasks ([source](./crates/sel4-root-task))
- A runtime for [seL4 Core Platform](https://github.com/BreakawayConsulting/sel4cp)
  protection domains ([source](./crates/sel4cp))
- A [CapDL](https://docs.sel4.systems/projects/capdl/)-based system initializer ([source and
  docs](./crates/sel4-capdl-initializer))
- A loader for the seL4 kernel ([source and docs](./crates/sel4-kernel-loader))
- Custom `rustc` target specifications for seL4 userspace ([JSON and docs](./support/targets))
- Many more crates for use in seL4 userspace

The [./hacking](./hacking) directory contains scripts for developing and testing these crates using
Nix and, optionally, Docker.

This work is funded by the [seL4 Foundation](https://sel4.systems/Foundation/home.pml).

### Rendered rustdoc

[https://coliasgroup.com/rust-seL4/](https://coliasgroup.com/rust-seL4/)

### Overview of crates

##### General crates

- [`sel4`](./crates/sel4): Straightforward, pure-Rust bindings to the seL4 API.
- [`sel4-sys`](./crates/sel4/sys): Raw bindings to the seL4 API, generated from the libsel4 headers
  and interface definition files. This crate is not intended to be used directly by application
  code, but rather serves as a basis for the `sel4` crate's implementation.
- [`sel4-config`](./crates/sel4/config): Macros and constants corresponding to the seL4 kernel
  configuration. Can be used by all targets (i.e. in all of: application code, build scripts, and
  build-time tools).
- [`sel4-platform-info`](./crates/sel4-platform-info): Constants corresponding to the contents of
  `platform_info.h`. Can be used by all targets.
- [`sel4-sync`](./crates/sel4-sync): Synchronization constructs using seL4 IPC. Currently only
  supports notification-based mutexes.
- [`sel4-logging`](./crates/sel4-logging): Log implementation for the
  [`log`](https://crates.io/crates/log) crate.
- [`sel4-externally-shared`](./crates/sel4-externally-shared): Abstractions for interacting with
  data structures in shared memory.
- [`sel4-shared-ring-buffer`](./crates/sel4-shared-ring-buffer): Implementation of shared data
  structures used in the [seL4 Device Driver Framework](https://github.com/lucypa/sDDF).
- [`sel4-async-*`](./crates/sel4-async): Crates for leveraging async Rust in seL4 userspace.

##### Runtime crates

- **Root task**:
  - [`sel4-root-task`](./crates/sel4-root-task): A runtime for root tasks that supports thread-local
    storage and unwinding, and provides a global allocator.
- **seL4 Core Platform**:
  - [`sel4cp`](./crates/sel4cp): A runtime for [seL4 Core
    Platform](https://github.com/BreakawayConsulting/sel4cp) protection domains, including an
    implementation of libsel4cp and abstractions for IPC.

##### Programs

- [`sel4-capdl-initializer`](./crates/sel4-capdl-initializer): A
  [CapDL](https://docs.sel4.systems/projects/capdl/)-based system initializer.
- [`sel4-kernel-loader`](./crates/sel4-kernel-loader): A loader for the seL4 kernel, similar in
  purpose to [elfloader](https://github.com/seL4/seL4_tools/tree/master/elfloader-tool).

### Integrating these crates into your project

These crates are not yet hosted on [crates.io](https://crates.io). Use them either as Git or path
Cargo dependencies.

Some of these crates depend, at build time, on external components and configuration. In all cases,
information for locating these dependencies is passed to the dependant crates via environment
variables which are interpreted by `build.rs` scripts. Here is a list of environment variables that
the crates which use them:

- `sel4-config` and `sel4-sys`, whose dependants include `sel4`, `sel4-root-task`, `sel4cp`, and
  many more, use `$SEL4_INCLUDE_DIRS` (defaulting to `$SEL4_PREFIX/libsel4/include` if
  `$SEL4_PREFIX` is set) which must contain a colon-separated list of include paths for the libsel4
  headers. See the the `sel4` crate's rustdoc for more information.
- `sel4-platform-info`, whose dependants include `sel4-kernel-loader`, uses `$SEL4_PLATFORM_INFO`
  (defaulting to `$SEL4_PREFIX/support/platform_gen.yaml` if `$SEL4_PREFIX` is set) which must
  contain the path of the `platform_gen.yaml` file from the seL4 kernel build system.
- `sel4-kernel-loader` uses `$SEL4_KERNEL` (defaulting to `$SEL4_PREFIX/bin/kernel.elf` if
  `$SEL4_PREFIX` is set) which must contain the path of the seL4 kernel (as an ELF executable).
  Furthermore, if `$SEL4_KERNEL_LOADER_CONFIG` is set, then `sel4-kernel-loader` overrides the
  default configuration with one in the provided JSON file. Note that no configuration options are
  actually implemented yet.

The best way to learn how to integrate these crates into your project is to check out these concrete
examples of their use:

- Simple root task: https://github.com/coliasgroup/rust-seL4-root-task-demo
- Using the seL4 Core Platform: https://github.com/coliasgroup/rust-seL4-sel4cp-demo

### Quick start for running the tests in this repository

The only requirements for running the tests in this repository are Git, Make, and Docker.

First, clone this repository:

```
git clone https://github.com/coliasgroup/rust-seL4
cd rust-seL4
```

Next, build, run, and enter a Docker container for development:

```
cd hacking/docker && make run && make exec
```

Inside the container at the repository's top-level directory, build and simulate a simple seL4-based
system with a [root task](./crates/examples/root-task/example-root-task) written in Rust (this will
take a few minutes):

```
make example
```

Also inside the container at the repository's top-level directory, build and run all of this
repository's automated tests:

```
make run-tests
```
