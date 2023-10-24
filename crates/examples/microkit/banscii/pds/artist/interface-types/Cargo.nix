#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, serdeWith }:

mk {
  package.name = "banscii-artist-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
}
