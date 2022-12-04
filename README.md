# Rust support for seL4 userspace

This repository contains crates for seL4 userspace.

### Quick start

First, clone this respository:

```
git clone https://gitlab.com/coliasgroup/rust-seL4
cd rust-seL4
```

Next, build, run, and enter a Docker container for development:

```
make -C ./docker run && make -C ./docker exec
```

Finally, inside the container, build and emulate a simple seL4-based system with a root task written in Rust:

```
make example
```
