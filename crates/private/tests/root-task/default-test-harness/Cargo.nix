#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-root-task-default-test-harness";
  dev-dependencies = {
    test = let package = "sel4-root-task-default-test-harness"; in localCrates.${package} // {
      inherit package;
    };
  };
}
