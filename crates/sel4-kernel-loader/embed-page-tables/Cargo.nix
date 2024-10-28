#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-kernel-loader-embed-page-tables";
  dependencies = {
    inherit (versions) proc-macro2 quote bitfield;
  };
}
