<!--
     Copyright 2023, Colias Group, LLC

     SPDX-License-Identifier: CC-BY-SA-4.0
-->

# Cargo manifest management tools

This project contains over 100 crates. Mantaining the consistency of all of the associated Cargo
manifest files would be tedious and error prone. For the sake of both developer productivity and
correctness, we automate the management of the Cargo manifest files in the project with the tools in
this directory. In particular, these tools allow us to express the contents of Cargo manifests much
more concisely and at a higher level of abstraction in `Cargo.nix` files, from which corresponding
`Cargo.toml` files are generated.

No other aspects of this project depend on the use of this tool. You can manually modify the
generated `Cargo.toml` files and use these crates in another project, or even run the tests in this
project using Nix. However, the "Check sources" job in
[.github/workflows/push.yaml](../../.github/workflows/push.yaml) does assume that this tool is being
used. In particular, it requires `Cargo.toml` files to be consistent with their adjacent `Cargo.nix`
files.

Concrete benefits of using this tool include:
- Not having to worry about manually declaring and mantaining the consistency of mostly uniform
  package metadata such as edition, licenses, authors, etc.
- The ability to refer to local crates symbolically. This eliminates the need to manually mantain
  the accuracy of relative paths between crates and, when relevant, the versions of these crates.
- The ability to refer to version bounds and Git sources of remote dependencies symbolically. Using
  consistent version bounds and Git sources for each instance of a given remote dependency
  throughout the project enables proper dependency resolution. Referring to these symbolically makes
  this less tedious, and also makes updating dependencies in an intentional way easy.

You don't need to have experience with Nix to create and modify `Cargo.nix` files. They are written
using the Nix programming language, but they don't depend on any of the advanced features of Nixpkgs
or the Nix package manager. You can think of them as similar JSON files with variables and
functions. See the [Nix Language](https://nixos.org/manual/nix/unstable/language/index.html) section
of the Nix Reference Manual for syntax.

Each `Cargo.nix` file is a function from an attribute set (like a JSON object) to an attribute set.
The input attribute set is like a set of imports, and the output attribute set is the content of a
Cargo manifest. The function is called with arguments from
[./manifest-scope.nix](./manifest-scope.nix), plus a special argument called `localCrates`. The
function's result is the value of a Cargo manifest (i.e. an attribute set with keys like `package`
and `dependencies`), optionally with a special attribute called `nix`. The `nix` attribute is not
included in the resulting `Cargo.toml` file, and is instead used to pass meta information to these
tools.

Here is an example `Cargo.nix`:

`Cargo.nix`:
```nix
{ myFavoriteEdition, myFavoriteLogVersion }:

{
  package.name = "foo";
  package.version = "1.2.3";
  package.edition = myFavoriteEdition;
  dependencies = {
    log = { version = myFavoriteLogVersion; default-features = false; };
  }
}
```
`Cargo.toml`:
```toml
[package]
name = "foo"
version = "1.2.3"
edition = 2024

[dependencies]
log = { version = "0.4", default-features = false }
```

From within this directory, `make update` generates `Cargo.toml` files from `Cargo.nix` files, and
overwrites stale `Cargo.toml` with updated contents. `make check` generates `Cargo.toml` files from
`Cargo.nix`, but does not modify any existing in-tree `Cargo.toml` files. Instead, it fails if any
existing in-tree `Cargo.toml` file is stale. `make check` can be used to determine whether running
`make update` is necessary.

From the top-level directory of this project, `make update-generated-sources` and `make
check-generated-sources` invoke the corresponding target in this directory, and also update or check
the top-level `Cargo.lock`.

### The special `localCrates` argument

`localCrates` is an attribute set which maps local crate names to partial Cargo manifest dependency
tables. Currently, these tables only include the `path` attribute. Paths are relative to the current
`Cargo.nix`. Thus, the value of the `localCrates` argument depends on the current `Cargo.nix` file.
For example, in [`../../crates/sel4-microkit/Cargo.nix`](../../crates/sel4-microkit/Cargo.nix), the
value of `localCrates` is:

```nix
{
  sel4 = { path = "../sel4"; };
  sel4-sys = { path = "../sel4/sys"; };
  # ...
}
```

This allows for the following:

`Cargo.nix`:
```nix
{ localCrates }:

{
  package.name = "sel4-microkit";
  package.version = "1.2.3";
  dependencies = {
    log = "0.4";
    inherit (localCrates) sel4;
    sel4-sync = localCrates.sel4-sync // {
      default-features = false;
    }
  }
}
```
`Cargo.toml`:
```toml
[package]
name = "sel4-microkit"
version = "1.2.3"

[dependencies]
log = "0.4"
sel4 = { path = "../sel4" }
sel4-sync = { path = "../sel4-sync" }
```

### The special `nix` output attribute

The `nix` output attribute of a `Cargo.toml` file is optional. If present, it should be an attribute
set with the following optional attributes:
- `frontmatter: str`
- `justCheckForEquivalence: bool`

`frontmatter` will be prepended to the resulting `Cargo.toml`. It is meant to be used for comments.
For example:

`Cargo.nix`:
```nix
{ }:

{
  nix.frontmatter = ''
    # Foo
    # Bar
  '';
  package.name = "foo";
  package.version = "1.2.3";
}
```
`Cargo.toml`:
```toml
# Foo
# Bar
[package]
name = "foo"
version = "1.2.3"
```

`justCheckForEquivalence` means that the adjacent `Cargo.toml` isn't generated from `Cargo.nix`, but
rather just checked for structural equivalence. This allows for the manually-written `Cargo.toml` to
be formatted or annotated in ways not supported by these tools (e.g. with comments throughout), but
for that manifest to still benefit from consistency checks. This is an advanced feature.

### Arguments provided by [./manifest-scope.nix](./manifest-scope.nix)

[./manifest-scope.nix](./manifest-scope.nix) is meant to be read and modified by users of this tool.
It is an attribute set whose values are available to `Cargo.nix` files.

`TODO: document some of these attributes`
