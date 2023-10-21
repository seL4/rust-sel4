#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "banscii-assistant-core-test";
  dependencies = {
    env_logger = "0.10.0";
    inherit (versions) log;
  };
  nix.local.dependencies = with localCrates; [
    banscii-assistant-core
  ];
}
