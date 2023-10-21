<!--
     Copyright 2023, Colias Group, LLC

     SPDX-License-Identifier: CC-BY-SA-4.0
-->

The public APIs of this project's crates depend heavily on the combination of seL4 configuration,
crate feature selection, and rustc target specification. We have generated rustdoc for a few
representative combinations. Each entry below provides a consistent view of all relevant crates for
one such combination.

The rustdoc for each view is generated all at once with one `cargo doc` invocation on the
[`meta`](https://github.com/seL4/rust-sel4/tree/main/crates/private/meta) crate, whose only
purpose is to depend on and select features for the other crates. Due to a current limitation of
rustdoc, each view can only include at most one language runtime crate (e.g. `sel4-root-task` or
`sel4-microkit`).
