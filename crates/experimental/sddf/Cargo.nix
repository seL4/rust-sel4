#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sddf";
  dependencies = {
    ptr_meta = { version = versions.ptr_meta; default-features = false; };
    num_enum = { version = versions.num_enum; default-features = false; };
    inherit (localCrates)
      sddf-sys
      sel4-config
      sel4-immutable-cell
      sddf-ipc-types
    ;
    sel4-shared-memory = localCrates.sel4-shared-memory // {
      features = [ "atomics" ];
    };
    sel4-microkit-base = localCrates.sel4-microkit-base // {
      optional = true;
    };
  };
  features = {
    "sel4-microkit-base" = [ "dep:sel4-microkit-base" "sddf-ipc-types/sel4-microkit-base" ];
  };
}
