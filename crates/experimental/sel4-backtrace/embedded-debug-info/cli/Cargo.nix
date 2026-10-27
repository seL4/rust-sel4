#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, objectWith }:

mk {
  package.name = "sel4-backtrace-embedded-debug-info-cli";
  dependencies = {
    object = objectWith [];
    clap = { version = versions.clap; features = [ "derive" ]; };
    inherit (localCrates) sel4-patch-elf sel4-phdrs-constants;
  };
}
