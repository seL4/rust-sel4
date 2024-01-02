#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "tests-root-task-default-test-harness";
  dependencies = {
    inherit (versions) log;
  };
  dev-dependencies = {
    test = let package = "sel4-root-task-default-test-harness"; in localCrates.${package} // {
      inherit package;
    };
  };
}
