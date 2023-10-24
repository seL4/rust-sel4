#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "cargo-helpers";
  dependencies = {
    inherit (versions) clap;
    cargo-util = "0.2.3";
    cargo = "0.73.1";
  };
}
