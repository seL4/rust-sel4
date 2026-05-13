#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions  }:

mk {
  package.name = "sel4-backtrace-embedded-debug-info-cli";
  dependencies = {
    inherit (versions) object;
    clap = { version = versions.clap; features = [ "derive" ]; };
    inherit (localCrates) sel4-patch-elf sel4-phdrs-constants;
  };
}
