#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, postcardWith, versions }:

mk {
  package.name = "sel4-capdl-initializer";
  dependencies = {
    inherit (versions)
      log
      one-shot-mutex
    ;
    postcard = postcardWith [ "alloc" ];
    rkyv = { version = versions.rkyv; default-features = false; };
    inherit (localCrates)
      sel4-capdl-initializer-core
      sel4
      sel4-dlmalloc
      sel4-logging
    ;
    sel4-root-task = localCrates.sel4-root-task // { features = [ "single-threaded" ]; };
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // { features = [ "serde" "deflate" ]; };
  };
  # features = {
  #   deflate = [
  #     "sel4-capdl-initializer-types/deflate"
  #   ];
  # };
}
