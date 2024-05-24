#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

let
  topLevelDir = ../../../../../..;
  topLevel = import topLevelDir {};

in
topLevel.worlds.default.shell.overrideAttrs (self: super: {
  nativeBuildInputs = (super.nativeBuildInputs or []) ++ [
    topLevel.pkgs.build.dafny
  ];

  TOP_LEVEL_DIR = toString topLevelDir;
  TRANSLATED = toString ./build/translated.rs;
})
