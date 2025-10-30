#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-microkit";
  dependencies = {
    inherit (versions)
      cfg-if
      one-shot-mutex
    ;
    inherit (localCrates)
      sel4-immediate-sync-once-cell
      sel4-panicking-env
      sel4-panicking
      sel4-dlmalloc
      sel4-microkit-base
      sel4-microkit-macros
    ;
    sel4-runtime-common = localCrates.sel4-runtime-common // { features = [ "abort" ]; };
    sel4 = localCrates.sel4 // { features = [ "single-threaded" ]; };
  };
  features = {
    full = [
      "alloc"
    ];
    alloc = [
      "sel4-panicking/alloc"
    ];
  };
}
