#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith, postcardWith }:

mk {
  package.name = "sel4-simple-task-rpc";
  dependencies = {
    serde = serdeWith [];
    postcard = postcardWith [];
    inherit (localCrates) sel4;
  };
}
