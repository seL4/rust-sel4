#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, objectWith }:

mk {
  package.name = "sel4-reset-cli";
  dependencies = {
    inherit (versions)
      anyhow
      rangemap
    ;
    object = objectWith [];
    clap = { version = versions.clap; features = [ "derive" ]; };
    inherit (localCrates)
      sel4-patch-elf
      sel4-phdrs-constants
    ;
  };
}
