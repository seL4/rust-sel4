#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, postcardWith }:

mk {
  package.name = "sel4-capdl-initializer-add-spec";
  dependencies = {
    inherit (versions)
      anyhow
      serde_json
      num
      clap
    ;
    object = { version = versions.object; features = [ "all" ]; };
    postcard = postcardWith [ "alloc" ];
    inherit (localCrates)
      sel4-synthetic-elf
    ;
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // { features = [ "std" "serde" "deflate" ]; };
  };
}
