#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-capdl-initializer-with-embedded-spec-embedded-spec-validate";
  features = {
    deflate = [ "sel4-capdl-initializer-with-embedded-spec-embedded-spec/deflate" ];
  };
  dependencies = {
    inherit (localCrates)
      sel4-capdl-initializer-types
      sel4-capdl-initializer-with-embedded-spec-build-env
      sel4-capdl-initializer-with-embedded-spec-embedded-spec
    ;
  };
}
