#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-capdl-initializer-with-embedded-spec";
  dependencies = {
    inherit (localCrates)
      sel4-capdl-initializer-core
      sel4-capdl-initializer-with-embedded-spec-embedded-spec
      sel4-capdl-initializer-types
      sel4
      sel4-logging
    ;
    sel4-root-task = localCrates.sel4-root-task // { features = [ "single-threaded" ]; };
  };
  build-dependencies = {
    inherit (localCrates)
      sel4-capdl-initializer-with-embedded-spec-embedded-spec-validate
    ;
  };
  features = {
    deflate = [
      "sel4-capdl-initializer-with-embedded-spec-embedded-spec/deflate"
      "sel4-capdl-initializer-with-embedded-spec-embedded-spec-validate/deflate"
    ];
  };
}
