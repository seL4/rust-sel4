#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "spawn-task-child";
  dependencies = {
    inherit (versions)
      cfg-if
      one-shot-mutex
    ;
    inherit (localCrates)
      sel4
      sel4-panicking-env
      sel4-dlmalloc
    ;
    sel4-panicking = localCrates.sel4-panicking // {
      features = [ "personality" "panic-handler" "alloc" ];
    };
    sel4-runtime-common = localCrates.sel4-runtime-common // {
      features = [ "full" ];
    };
  };
}
