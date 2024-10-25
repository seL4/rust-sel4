#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer-embed-spec";
  dependencies = {
    inherit (versions)
      proc-macro2
      quote
      syn
      serde
      serde_json
    ;
    hex = "0.4.3";
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // { features = [ "serde" "std" "deflate" ]; };
  };
}
