#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, runCommand

, dafny

, sources
, crates
, crateUtils
, seL4Modifications
, mkTask

, mkInstance

, canSimulate
}:

let
  verifiedSrcDir = lib.cleanSource (sources.srcRoot + "/crates/private/tests/root-task/dafny/verified");
  verifiedName = "core";
  verifiedSrc = "${verifiedSrcDir}/${verifiedName}.dfy";

  translated = runCommand "translated.rs" {
    nativeBuildInputs = [ dafny ];
  } ''
	  dafny translate rs ${verifiedSrc} -o ${verifiedName}
    mv ${verifiedName}-rust/src/${verifiedName}.rs $out
  '';
      # TODO
      # --standard-libraries \

in
mkInstance {
  rootTask = mkTask rec {
    rootCrate = crates.tests-root-task-dafny-task;
    release = false;

    layers = [
      crateUtils.defaultIntermediateLayer
      {
        crates = [ "sel4-root-task" ];
        modifications = seL4Modifications;
      }
    ];

    lastLayerModifications.modifyDerivation = drv: drv.overrideAttrs (attrs: {
      nativeBuildInputs = (attrs.nativeBuildInputs or []) ++ [
        dafny
      ];

      TRANSLATED = translated;
    });
  };

  extraPlatformArgs = lib.optionalAttrs canSimulate  {
    canAutomateSimply = true;
  };
} // {
  inherit translated;
}
