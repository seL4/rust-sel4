#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib

, crates
, crateUtils
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

    verifyWithVerus = true;

    layers = [
      crateUtils.defaultIntermediateLayer
      {
        crates = [ "sel4-root-task" ];
        modifications = seL4Modifications;
      }
    ];
  };

  extraPlatformArgs = lib.optionalAttrs canSimulate  {
    canAutomateSimply = true;
  };
}
