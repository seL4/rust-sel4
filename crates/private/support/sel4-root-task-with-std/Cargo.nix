#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-root-task-with-std";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-panicking-env
    ;
    sel4-runtime-common = localCrates.sel4-runtime-common // { features = [ "sel4" ]; };
  };
  features = {
    single-threaded = [
      "sel4/single-threaded"
    ];
  };
}
