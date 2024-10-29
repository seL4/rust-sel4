#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, dafnySource }:

mk {
  package.name = "tests-root-task-dafny-core";
  dependencies = {
    inherit (localCrates)
      sel4-mod-in-out-dir
      # dafny_runtime
    ;
    dafny_runtime = dafnySource;
    num = { version = versions.num; default-features = false; features = ["alloc"]; };
  };
}
