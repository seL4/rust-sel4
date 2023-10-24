#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, serdeWith }:

mk {
  package.name = "microkit-http-server-example-sp804-driver-interface-types";
  dependencies = {
    serde = serdeWith [];
  };
}
