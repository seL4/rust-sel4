#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-microkit-passive-server-with-deferred-action-pds-client";
  dependencies = {
    inherit (localCrates)
      sel4-microkit
    ;
  };
}
