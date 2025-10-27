#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-config-data";
  dependencies = {
    inherit (versions) serde_json;
    sel4-config-types = localCrates.sel4-config-types // { features = [ "serde" ]; };
  };
  build-dependencies = {
    inherit (versions) serde_json;
    inherit (localCrates) sel4-build-env;
    sel4-config-types = localCrates.sel4-config-types // { features = [ "serde" ]; };
  };
}
