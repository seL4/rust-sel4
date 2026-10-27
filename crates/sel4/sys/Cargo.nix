#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-sys";
  package.links = "sel4";
  package.build = "build/main.rs";
  dependencies = {
    inherit (versions) log;
    inherit (localCrates)
      sel4-config
      sel4-bitfield-ops
    ;
  };
  build-dependencies = {
    inherit (versions) glob;
    proc-macro2 = { version = versions.proc-macro2; default-features = false; };
    quote = { version = versions.quote; default-features = false; };
    bindgen = { version = versions.bindgen; default-features = false; features = [ "runtime" ]; };
    nom = { version = versions.nom; default-features = false; features = [ "alloc" ]; };
    roxmltree = { version = versions.roxmltree; default-features = false; };
    inherit (localCrates)
      sel4-build-env
      sel4-config
      sel4-config-data
    ;
  };
}
