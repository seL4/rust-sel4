#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-platform-info";
  dependencies = {
    inherit (localCrates)
      sel4-platform-info-types
    ;
  };
  build-dependencies = {
    inherit (versions) proc-macro2 quote serde_yaml;
    serde = { version = versions.serde; features = [ "derive" ]; };
    inherit (localCrates)
      sel4-build-env
      sel4-config
    ;
  };
}
