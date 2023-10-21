#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "example-root-task-without-runtime";
  dependencies = {
    sel4.default-features = false;
    inherit (versions) cfg-if;
  };
  nix.local.dependencies = with localCrates; [
    sel4
  ];
}
