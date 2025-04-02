#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, serdeWith, postcardWith }:

mk {
  package.name = "sel4-ipc-types";
  dependencies = {
    zerocopy = {
      version = versions.zerocopy;
      optional = true;
    };
    serde = serdeWith [] // {
      optional = true;
    };
    postcard = postcardWith [] // {
      optional = true;
    };
    sel4-microkit-base = localCrates.sel4-microkit-base // {
      optional = true;
    };
  };
  features = {
    postcard = [ "dep:postcard" "dep:serde" ];
  };
}
