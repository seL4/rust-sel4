#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, haskell
, sources
}:

let
  haskellPackages = haskell.packages.ghc9103.override {
    overrides = self: super: {
      capDL-tool = self.callPackage ./capDL-tool.nix {
        inherit sources;
      };
    };
  };

in
haskellPackages.capDL-tool
