#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-test-runner";
  dependencies = {
    inherit (versions)
      anyhow
      tempfile
      object
    ;
    clap = { version = versions.clap; features = [ "derive" ]; };
    inherit (localCrates)
      sel4-test-sentinels-wrapper
    ;
  };
}
