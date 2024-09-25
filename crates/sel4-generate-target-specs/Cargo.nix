#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-generate-target-specs";
  dependencies = {
    inherit (versions) serde_json clap;
  };
  build-dependencies = {
    inherit (versions) rustc_version;
  };
  package.metadata.rust-analyzer = {
    rustc_private = true;
  };
}
