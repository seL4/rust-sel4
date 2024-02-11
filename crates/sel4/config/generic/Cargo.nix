#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-config-generic";
  dependencies = {
    inherit (versions)
      fallible-iterator
      proc-macro2
      quote
    ;
    syn = { version = versions.syn; features = [ "full" ]; };
    inherit (localCrates)
      sel4-config-generic-types
    ;
  };
}
