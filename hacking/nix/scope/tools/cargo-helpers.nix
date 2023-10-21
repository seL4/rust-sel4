#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, buildCrateInLayersHere
, crateUtils
, crates
, pkgconfig
, openssl
}:

buildCrateInLayersHere {
  rootCrate = crates.cargo-helpers;
  release = false;

  commonModifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      nativeBuildInputs = (super.nativeBuildInputs or []) ++ [
        pkgconfig
      ];
      buildInputs = (super.buildInputs or []) ++ [
        openssl
      ];
    });
  };
}
