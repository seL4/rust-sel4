#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "tests-root-task-panicking";
  dependencies = {
    inherit (versions) cfg-if;
  };
  features = {
    alloc = [ "sel4-root-task/alloc" ];
    panic-unwind = [];
    panic-abort = [];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task
  ];
}
