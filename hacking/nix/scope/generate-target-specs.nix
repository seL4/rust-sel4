#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, makeWrapper
, buildCratesInLayers
, crateUtils
, crates
}:

{ rustEnvironment }:

buildCratesInLayers rec {
  inherit rustEnvironment;
  rootCrate = crates.sel4-generate-target-specs;
  lastLayerModifications = crateUtils.elaborateModifications {
    # HACK
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      nativeBuildInputs = (super.nativeBuildInputs or []) ++ [ makeWrapper ];
      postBuild = ''
        wrapProgram $out/bin/${rootCrate.name} \
          --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ rustEnvironment.rustToolchain ]}
      '';
    });
  };
}
