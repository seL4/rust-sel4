#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, postcardWith, versions }:

mk {
  package.name = "sel4-capdl-initializer";
  dependencies = {
    inherit (versions) log;
    postcard = postcardWith [ "alloc" ];
    inherit (localCrates)
      sel4-capdl-initializer-core
      sel4
      sel4-dlmalloc
      sel4-logging
      sel4-sync
    ;
    sel4-root-task = localCrates.sel4-root-task // { default-features = false; features = [ "alloc" "single-threaded" ]; };
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // { features = [ "alloc" "serde" "deflate" ]; };
  };
  # features = {
  #   deflate = [
  #     "sel4-capdl-initializer-types/deflate"
  #   ];
  # };
}
