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
      clap
      rkyv
    ;
    object = { version = versions.object; features = [ "all" ]; };
    inherit (localCrates)
      sel4-synthetic-elf
    ;
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // {
      features = [
        "serde"
        "deflate"
        "fill-utils"
      ];
    };
  };
}
