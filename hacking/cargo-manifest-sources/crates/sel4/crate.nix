#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4";
  package.license = "MIT";
  dependencies = {
    inherit (versions) cfg-if;
  };
  features = {
    default = [ "state" ];
    state = [];
    single-threaded = [];
  };
  nix.local.dependencies = with localCrates; [
    sel4-config
    sel4-sys
  ];
}
