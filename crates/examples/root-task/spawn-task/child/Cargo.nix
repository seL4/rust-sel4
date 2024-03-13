#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "spawn-task-child";
  dependencies = {
    inherit (versions) cfg-if;
    inherit (localCrates)
      sel4
      sel4-panicking-env
      sel4-dlmalloc
      sel4-sync
    ;
    sel4-panicking = localCrates.sel4-panicking // {
      features = [ "unwinding" "alloc" ];
    };
    sel4-runtime-common = localCrates.sel4-runtime-common // {
      features = [ "start" "tls" "unwinding" ];
    };
  };
}
