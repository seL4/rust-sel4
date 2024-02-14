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
  haskellPackages = haskell.packages.ghc928.override {
    overrides = self: super: {
      base-compat = self.callPackage ./base-compat-0-12-2.nix {};
      base-compat-batteries = self.callPackage ./base-compat-batteries-0-12-2.nix {};
      capDL-tool = self.callPackage ./capDL-tool.nix {
        inherit sources;
      };
    };
  };

in
haskellPackages.capDL-tool
