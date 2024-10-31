#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-root-task";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-immediate-sync-once-cell
      sel4-panicking
      sel4-panicking-env
      sel4-dlmalloc
      sel4-sync
      sel4-ctors-dtors
      sel4-root-task-macros
    ;
    sel4-runtime-common = localCrates.sel4-runtime-common // { features = [ "tls" "start" ]; };
  };
  features = {
    full = [
      "alloc"
    ];
    alloc = [
      "sel4-panicking/alloc"
    ];
    single-threaded = [
      "sel4/single-threaded"
    ];
  };
}
