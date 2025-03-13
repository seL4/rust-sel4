#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, postcardWith, serdeWith }:

mk {
  package.name = "sel4-kernel-loader-add-payload";
  dependencies = {
    inherit (versions)
      anyhow
      serde_json
      serde_yaml
      heapless
      num
      clap
    ;
    object = { version = versions.object; features = [ "all" ]; };
    postcard = postcardWith [ "alloc" ];
    serde = serdeWith [ "alloc" "derive" ];
    inherit (localCrates)
      sel4-kernel-loader-config-types
      sel4-synthetic-elf
    ;
    sel4-kernel-loader-payload-types = localCrates.sel4-kernel-loader-payload-types // { features = [ "serde" ]; };
    sel4-config-types = localCrates.sel4-config-types // { features = [ "serde" ]; };
  };
}
