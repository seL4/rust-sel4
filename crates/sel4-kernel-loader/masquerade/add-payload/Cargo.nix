#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-kernel-loader-masquerade-add-payload";
  dependencies = {
    inherit (versions)
      anyhow
      object
    ;
    clap = { version = versions.clap; features = [ "derive" ]; };
  };
}
