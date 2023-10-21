#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, postcardWith, versions }:

mk {
  package.name = "sel4-capdl-initializer";
  dependencies = {
    sel4-capdl-initializer-types.features = [ "alloc" "serde" "deflate" ];
    postcard = postcardWith [ "alloc" ];
    sel4-root-task = { default-features = false; features = [ "alloc" "single-threaded" ]; };
    inherit (versions) log;
  };
  # features = {
  #   deflate = [
  #     "sel4-capdl-initializer-types/deflate"
  #   ];
  # };
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-core
    sel4-capdl-initializer-types
    sel4
    sel4-dlmalloc
    sel4-logging
    sel4-root-task
    sel4-sync
  ];
}
