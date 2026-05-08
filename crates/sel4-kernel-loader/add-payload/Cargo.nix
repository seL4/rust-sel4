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
      num
      clap
      object
      rkyv
    ;
    serde = serdeWith [ "alloc" "derive" ];
    inherit (localCrates)
      sel4-patch-elf
      sel4-phdrs-constants
      sel4-kernel-loader-payload-types
    ;
    sel4-config-types = localCrates.sel4-config-types // { features = [ "serde" ]; };
  };
}
