#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer-with-embedded-spec-build-env";
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-embed-spec
    sel4-capdl-initializer-types
  ];
  dependencies = {
    sel4-capdl-initializer-types.features = [ "std" "serde" ];
    inherit (versions) serde serde_json;
  };
}
