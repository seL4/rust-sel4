#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "tests-root-task-musl";
  dependencies = {
    inherit (versions)
      dlmalloc
      one-shot-mutex
    ;
    inherit (localCrates)
      sel4
      sel4-root-task-with-std
      sel4-musl
      sel4-linux-syscall-types
      sel4-dlmalloc
    ;
  };
}
