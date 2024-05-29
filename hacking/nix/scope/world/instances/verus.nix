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

, mkInstance

, canSimulate
}:

mkInstance {
  rootTask = mkTask rec {
    rootCrate = crates.tests-root-task-verus-task;
    release = false;

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
