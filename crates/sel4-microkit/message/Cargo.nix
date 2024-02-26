#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith }:

mk {
  package.name = "sel4-microkit-message";
  dependencies = {
    serde = serdeWith [] // {
      optional = true;
    };
    inherit (localCrates)
      sel4-microkit-base
      sel4-microkit-message-types
    ;
  };
  features = {
    default = [ "postcard" ];
    postcard = [ "dep:serde" "sel4-microkit-message-types/postcard" ];
  };
}
