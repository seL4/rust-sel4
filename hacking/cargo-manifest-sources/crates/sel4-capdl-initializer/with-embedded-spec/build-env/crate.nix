#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer-with-embedded-spec-build-env";
  dependencies = {
    inherit (versions) serde serde_json;
    inherit (localCrates)
      sel4-capdl-initializer-embed-spec
    ;
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // { features = [ "std" "serde" ]; };
  };
}
