#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer-add-spec";
  dependencies = {
    inherit (versions)
      anyhow
      serde_json
      num
      rkyv
      object
    ;
    clap = { version = versions.clap; features = [ "derive" ]; };
    inherit (localCrates)
      sel4-patch-elf
      sel4-phdrs-constants
    ;
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // {
      features = [
        "serde"
        "deflate"
        "transform"
      ];
    };
  };
}
