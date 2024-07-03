#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-reset-cli";
  dependencies = {
    inherit (versions)
      anyhow
      fallible-iterator
      num
      clap
    ;
    object = { version = versions.object; features = [ "all" ]; };
    inherit (localCrates)
      sel4-render-elf-with-data
    ;
  };
}
