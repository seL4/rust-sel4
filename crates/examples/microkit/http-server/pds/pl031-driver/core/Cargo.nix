#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "microkit-http-server-example-pl031-driver-core";
  dependencies = {
    inherit (versions) log;
    tock-registers = "0.8.1";
  };
}
