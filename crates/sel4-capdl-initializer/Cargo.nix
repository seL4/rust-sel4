#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer";
  dependencies = {
    inherit (versions)
      log
    ;
    rkyv = { version = versions.rkyv; default-features = false; };
    inherit (localCrates)
      sel4
      sel4-logging
      sel4-immutable-cell
    ;
    sel4-root-task = localCrates.sel4-root-task // { features = [ "single-threaded" ]; };
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // { features = [ "sel4" ]; };
  };
  features = {
    default = [
      "alloc"
      "deflate"
    ];
    alloc = [];
    deflate = [
      "sel4-capdl-initializer-types/deflate"
    ];
  };
}
