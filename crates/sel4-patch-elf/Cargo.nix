#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, objectWith }:

mk {
  package.name = "sel4-patch-elf";
  dependencies = {
    object = objectWith [];
  };
}
