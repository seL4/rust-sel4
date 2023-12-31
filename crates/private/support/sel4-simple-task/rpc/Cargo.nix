#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith, postcardWith }:

mk {
  package.name = "sel4-simple-task-rpc";
  dependencies = {
    serde = serdeWith [] // { optional = true; };
    postcard = postcardWith [] // { optional = true; };
    inherit (localCrates) sel4;
  };
  features = {
    postcard = [
      "dep:serde"
      "dep:postcard"
    ];
  };
}
