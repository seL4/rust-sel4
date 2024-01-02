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
        "sel4-panicking-env"
      ];
    };
    getrandom = {
      version = "0.2.10";
      features = [ "custom" ];
    };
    rand = {
      version = "0.8.5";
      default-features = false;
      features = [ "small_rng" ];
    };
  };
}
