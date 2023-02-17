# Rust support for seL4 userspace

This repository contains crates for supporting the use of Rust in seL4 userspace. So far, this includes:

- Rust bindings to the seL4 API ([source](./crates/sel4))
- Crates supporting the construction of Rust language runtimes for seL4 userspace ([source](./crates/runtime))
- A limited loader for the seL4 kernel ([source](./crates/loader))

The [./hacking](./hacking) directory contains code for building and testing these crates using Nix and, optionally, Docker. However, the crates in this repository are in no way bound to that build system code.

Note that, for now, these crates depend on some patches to libsel4 which can be found at [coliasgroup/seL4:rust](https://gitlab.com/coliasgroup/seL4/-/tree/rust).

### Rendered rustdoc

[https://coliasgroup.gitlab.io/rust-seL4-html/](https://coliasgroup.gitlab.io/rust-seL4-html/)

### Overview of crates

##### Application-facing crates

- [`sel4`](./crates/sel4): Straightforward, pure-Rust bindings to the seL4 API.
- [`sel4-config`](./crates/sel4/config): Macros and constants corresponding to the seL4 kernel configuration. Can be used by all targets (i.e. in all of: application code, build scripts, and build-time tools).
- [`sel4-platform-info`](./crates/sel4/platform-info): Constants corresponding to the contents of `platform_info.h`. Can be used by all targets.
- [`sel4-sync`](./crates/runtime/sel4-sync): Synchronization constructs using seL4 IPC. Currently only supports notification-based mutexes.
- [`sel4-logging`](./crates/runtime/sel4-logging): Log implementation for the [`log`](https://crates.io/crates/log) crate.

##### Root task runtime

- [`sel4-root-task-runtime`](./crates/runtime/sel4-root-task-runtime): A featureful runtime which supports thread-local storage and unwinding, and provides a global allocator. 

##### Build system-facing crates

- [`loader`](./crates/loader): A loader for the seL4 kernel, similar in purpose to [elfloader](https://github.com/seL4/seL4_tools/tree/master/elfloader-tool).

##### Other crates of interest

- [`sel4-sys`](./crates/sel4/sys): Raw bindings to the seL4 API, generated from the libsel4 headers and interface definition files. The `sel4` crate's implementation is based on this crate.

### Integrating these crates into your project

The best way to learn how to integrate these crates into your project is to check out this concrete example of their use in a project with a simple build system:

https://gitlab.com/coliasgroup/rust-seL4-demos/simple-build-system-demo

Many of these crates depend, at build time, on external components and configuration.
In all cases, information about these dependencies is passed to the dependant crates via environment variables which are interpreted by `build.rs` scripts.
Here is a list of environment variables and the crates which use them:

- For crates in [`./crates/sel4`](./crates/sel4).
  See the rustdoc for the `sel4` crate for more information.
  See the `sel4-build-env` crate and its dependencies for implementation details.
    - `$SEL4_CONFIG`, defaulting to `$SEL4_PREFIX/support/config.json` if `$SEL4_PREFIX` is set:
      Must contain the path of a JSON representation of a seL4 kernel configuration.
      Required by the `sel4-config`, whose dependencies include the `sel4-sys` and `sel4` crates.
    - `$SEL4_INCLUDE_DIRS`, defaulting to `$SEL4_PREFIX/libsel4/include` if `$SEL4_PREFIX` is set:
      Must contain a colon-separated list of include paths for the libsel4 headers.
      Required by the `sel4-sys` crate , whose dependencies include the `sel4` crate.
    - `$SEL4_PLATFORM_INFO`, defaulting to `$SEL4_PREFIX/support/platform-info.yaml` if `$SEL4_PREFIX` is set:
      Must contain the path of a `platform-info.yaml` file from the seL4 kernel build system.
      Required by the `sel4-platform-info` crate, whose dependencies include the `loader` crate.
- For crates in [`./crates/loader`](./crates/loader). See the `loader-build-env` crate and its dependencies for implementation details.
    - `$SEL4_KERNEL`, defaulting to `$SEL4_PREFIX/bin/kernel.elf` if `$SEL4_PREFIX` is set:
      Must contain the path of the seL4 kernel (as an ELF executable).
      Required by the `loader` crate.
    - `$SEL4_DTB`, defaulting to `$SEL4_PREFIX/support/kernel.dtb` if `$SEL4_PREFIX` is set:
      Must contain the path a DTB for use by userspace.
      Required by the `loader` crate.
      In the future, providing this DTB at build time will be optional.
    - `$SEL4_APP`:
      Must contain the path of the root task (as an ELF executable).
      Required by the `loader` crate.
    - `$SEL4_LOADER_CONFIG`:
      Must contain the path of a JSON representation of a loader configuration.
      Required by the `loader` crate.
      Note that configuration for the loader isn't actually implemented yet!
      This configuration should be an empty JSON object.

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

Also inside the container at the repository's top-level directory, build run all of this repository's automated tests:

```
make run-automated-tests
```
