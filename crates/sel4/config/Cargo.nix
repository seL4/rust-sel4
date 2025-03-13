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
    inherit (versions)
      prettyplease
      proc-macro2
      quote
    ;
    syn = { version = versions.syn; features = [ "parsing" ]; };
    inherit (localCrates)
      sel4-config-data
      sel4-config-generic-types
    ;
  };
}
