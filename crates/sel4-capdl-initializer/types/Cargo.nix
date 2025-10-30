#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, serdeWith }:

# TODO make rkyv::{Serialize, Deserialze} optional

mk {
  package.name = "sel4-capdl-initializer-types";
  dependencies = {
    miniz_oxide = { version = versions.miniz_oxide; default-features = false; features = [ "with-alloc" ]; optional = true; };
    serde = serdeWith [ "derive" "alloc" ] // { optional = true; };
    rkyv = { version = versions.rkyv; default-features = false; features = [ "alloc" "bytecheck" "pointer_width_32" ]; };
    inherit (localCrates)
      sel4-capdl-initializer-types-derive
    ;
    sel4 = localCrates.sel4 // { optional = true; default-features = false; };
  };
  features = {
    deflate = [ "dep:miniz_oxide" ];
    fill-utils = [ "deflate" ];
  };
}
