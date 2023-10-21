#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "sel4-capdl-initializer-types";
  dependencies = {
    miniz_oxide = { version = "0.6.2"; default-features = false; optional = true; };
    sel4 = { optional = true; default-features = false; };
    serde = serdeWith [ "derive" "alloc" ] // { optional = true; };
    serde_json = { version = versions.serde_json; optional = true; };
    inherit (versions) cfg-if log;
  };
  features = {
    std = [ "alloc" "serde_json" ];
    alloc = [ "miniz_oxide?/with-alloc" ];
    serde = [ "alloc" "dep:serde" ];
    deflate = [ "dep:miniz_oxide" ];
    borrowed-indirect = [];
  };
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-types-derive
    sel4
  ];
}
