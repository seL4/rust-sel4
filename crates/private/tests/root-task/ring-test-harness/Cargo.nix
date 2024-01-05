#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk rec {
  package.name = "tests-root-task-ring-test-harness";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-test-harness
    ;
    sel4-newlib = localCrates.sel4-newlib // {
      features = [
        "nosys"
        "all-symbols"
      ];
    };
    getrandom = {
      version = versions.getrandom;
      features = [ "custom" ];
    };
    rand = {
      version = versions.rand;
      default-features = false;
      features = [ "small_rng" ];
    };
  };
}
