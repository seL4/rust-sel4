#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "banscii-pl011-driver-core";
  dependencies = {
    inherit (versions) tock-registers;
  };
}
