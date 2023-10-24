#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, unwindingWith }:

mk {
  package.name = "sel4-runtime-common";
  dependencies = {
    inherit (versions) cfg-if;
    unwinding = unwindingWith [] // { optional = true; };
    sel4-initialize-tls-on-stack = localCrates.sel4-initialize-tls-on-stack // { optional = true; };
    sel4-sync = localCrates.sel4-sync // { optional = true; };
    sel4-dlmalloc = localCrates.sel4-dlmalloc // { optional = true; };
  };
  features = {
    tls = [ "dep:sel4-initialize-tls-on-stack" ];
    start = [];
    static-heap = [ "sel4-sync" "sel4-dlmalloc" ];
  };
}
