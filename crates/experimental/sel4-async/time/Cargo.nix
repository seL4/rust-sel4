#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, smoltcpWith }:

mk {
  package.name = "sel4-async-time";
  dependencies = {
    inherit (versions) log pin-project;
  };
}
