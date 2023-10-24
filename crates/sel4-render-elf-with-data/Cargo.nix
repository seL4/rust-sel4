#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-render-elf-with-data";
  dependencies = {
    object = { version = versions.object; features = [ "all" ]; };
    inherit (versions) anyhow fallible-iterator num;
  };
}
