#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-sys";
  package.build = "build/main.rs";
  dependencies = {
    inherit (versions) log;
    inherit (localCrates)
      sel4-config
      sel4-bitfield-ops
    ;
  };
  build-dependencies = {
    inherit (versions)
      proc-macro2 quote prettyplease
      bindgen xmltree glob
    ;
    syn = { version = versions.syn; features = [ "parsing" ]; };
    inherit (localCrates)
      sel4-build-env
      sel4-bitfield-parser
      sel4-config
      sel4-config-data
    ;
  };
}
