#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-sys-wrappers";
  lib.crate-type = [ "staticlib" ];
  dependencies = {
    sel4-sys = localCrates.sel4-sys // { features = [ "wrappers" ]; };
  };
}
