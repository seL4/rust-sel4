#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib

, crates
, crateUtils
, globalPatchSection
, seL4Modifications
, mkTask

, verus

, mkInstance

, canSimulate
}:

mkInstance {
  rootTask = mkTask rec {
    inherit (verus) rustEnvironment;

    rootCrate = crates.tests-root-task-verus-task;
    release = false;

    extraProfile = {
      panic = "abort";
    };

    layers = [
      # TODO single layer to work around https://github.com/verus-lang/verus/issues/2209
      # crateUtils.defaultIntermediateLayer
      # {
      #   crates = [ "sel4-root-task" ];
      #   modifications = seL4Modifications;
      # }
    ];
  
    # HACK
    commonModifications = {
      modifyManifest = lib.flip crateUtils.combineConfig {
        patch.crates-io = {
          inherit (globalPatchSection.crates-io) hashbrown;
        };
      };
    };

    verifyWithVerus = true;
  };

  extraPlatformArgs = lib.optionalAttrs canSimulate  {
    canAutomateSimply = true;
  };
}
