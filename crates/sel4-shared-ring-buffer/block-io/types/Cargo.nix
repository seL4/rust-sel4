#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, zerocopyWith }:

mk {
  package.name = "sel4-shared-ring-buffer-block-io-types";
  dependencies = {
    num_enum = { version = versions.num_enum; default-features = false; };
    zerocopy = zerocopyWith [ "derive" ];
    inherit (localCrates)
      sel4-shared-ring-buffer
    ;
  };
}
