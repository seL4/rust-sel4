#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-initialize-tls";
  dependencies = {
    inherit (versions) cfg-if;
    sel4-alloca = localCrates.sel4-alloca // { optional = true; };
  };
  features = {
    on-stack = [ "sel4-alloca" ];
    on-heap = [ "alloc" ];
    alloc = [];
  };
}
