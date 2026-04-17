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
      num
      rangemap
      object
    ;
    clap = { version = versions.clap; features = [ "derive" ]; };
  };
}
