#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, serdeWith, postcardWith }:

mk {
  package.name = "sel4-microkit-message-types";
  dependencies = {
    inherit (versions) zerocopy;
    serde = serdeWith [] // {
      optional = true;
    };
    postcard = postcardWith [] // {
      optional = true;
    };
  };
  features = {
    default = [ "postcard" ];
    postcard = [ "dep:postcard" "dep:serde" ];
  };
}
