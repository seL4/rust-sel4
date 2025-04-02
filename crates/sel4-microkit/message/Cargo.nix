#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, serdeWith, postcardWith }:

mk {
  package.name = "sel4-microkit-message";
  dependencies = {
    inherit (versions) zerocopy;
    serde = serdeWith [];
    postcard = postcardWith [];
    inherit (localCrates)
      sel4-microkit-base
    ;
  };
}
