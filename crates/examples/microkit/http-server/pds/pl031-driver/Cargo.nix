#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "microkit-http-server-example-pl031-driver";
  dependencies = {
    inherit (localCrates)
      sel4-microkit-message
      microkit-http-server-example-pl031-driver-core
      microkit-http-server-example-pl031-driver-interface-types
    ;
    sel4-microkit = localCrates.sel4-microkit // { default-features = false; };
  };
}
