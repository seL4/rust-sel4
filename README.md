# Rust support for seL4 userspace

This repository contains crates for supporting the use of Rust in seL4 userspace. So far, this includes:

- Rust bindings to the seL4 API ([source](./crates/sel4), [docs](https://coliasgroup.gitlab.io/rust-seL4-html/))
- A limited [loader](./crates/loader) for the seL4 kernel

This repository also contains some code for building and testing these crates using Nix, Make, and, optionally, Docker. However, the crates are in no way bound to this build system code.

Note that, for now, these crates depend on some patches to libsel4 which can be found at [coliasgroup/seL4:rust](https://gitlab.com/coliasgroup/seL4/-/tree/rust).

### Quick start

The only requirements for getting started are Git, Make, and Docker.

First, clone this respository:

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
