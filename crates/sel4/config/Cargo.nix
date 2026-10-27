#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-config";
  dependencies = {
    inherit (localCrates)
      sel4-config-macros
    ;
  };
  build-dependencies = {
    proc-macro2 = { version = versions.proc-macro2; default-features = false; };
    quote = { version = versions.quote; default-features = false; };
    inherit (localCrates)
      sel4-config-data
      sel4-config-types
    ;
  };
}
