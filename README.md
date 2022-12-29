# Rust support for seL4 userspace

This repository contains crates for supporting the use of Rust in seL4 userspace. So far, this includes:

- Rust bindings to the seL4 API ([source](./crates/sel4), [docs](https://coliasgroup.gitlab.io/rust-seL4-html/))
- Crates supporting the construction of Rust language runtimes for seL4 userspace ([source](./crates/runtime))
- A limited loader for the seL4 kernel ([source](./crates/loader))

This repository also contains some code for building and testing these crates using Nix, Make, and, optionally, Docker. However, the crates are in no way bound to this build system code.

Note that, for now, these crates depend on some patches to libsel4 which can be found at [coliasgroup/seL4:rust](https://gitlab.com/coliasgroup/seL4/-/tree/rust).

### Overview of crates

##### Application-facing crates

- [`sel4`](./crates/sel4)
- [`sel4-config`](./crates/sel4/config)
- [`sel4-platform-info`](./crates/sel4/platform-info)
- [`sel4-sync`](./crates/runtime/sel4-sync)
- [`sel4-logging`](./crates/runtime/sel4-logging)

##### Example root task runtimes

- [`sel4-minimal-root-task-runtime`](./crates/runtime/sel4-minimal-root-task-runtime)

##### Build system-facing crates

- [`loader`](./crates/loader)

##### Other crates of interest

- [`sel4-sys`](./crates/sel4/sys)

### Integrating these crates into your project

The best way to learn how to integrate these crates into your project is to check out this concrete example of their use in a project with a simple build system:

https://gitlab.com/coliasgroup/rust-seL4-simple-build-system-demo

### Running the tests in this repository (quick start)

The only requirements for running the tests in this repository are Git, Make, and Docker.

First, clone this repository:

```
git clone https://gitlab.com/coliasgroup/rust-seL4
cd rust-seL4
```

Next, build, run, and enter a Docker container for development:

```
make -C docker/ run && make -C docker/ exec
```

Finally, inside the container, build and emulate a simple seL4-based system with a root task written in Rust:

```
make example
```
