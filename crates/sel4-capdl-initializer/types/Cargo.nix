#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "sel4-capdl-initializer-types";
  dependencies = {
    inherit (versions) cfg-if;
    miniz_oxide = { version = versions.miniz_oxide; default-features = false; features = [ "with-alloc" ]; optional = true; };
    serde = serdeWith [ "derive" "alloc" ] // { optional = true; };
    serde_json = { version = versions.serde_json; optional = true; };
    rkyv = { version = versions.rkyv; default-features = false; features = [ "alloc" "bytecheck" "pointer_width_32" ]; };
    inherit (localCrates)
      sel4-capdl-initializer-types-derive
    ;
    sel4 = localCrates.sel4 // { optional = true; default-features = false; };
  };
  features = {
    std = [ "serde_json" ];
    serde_json = [ "dep:serde_json" "serde" ];
    deflate = [ "dep:miniz_oxide" ];
  };
}
