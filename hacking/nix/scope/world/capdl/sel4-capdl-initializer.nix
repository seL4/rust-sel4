#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand
, capdl-tool
, objectSizes
, mkTask, crates
, crateUtils
, seL4Modifications
, seL4RustTargetInfoWithConfig
}:

mkTask {

  rootCrate = crates.sel4-capdl-initializer;

  rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = true; };

  release = true;

  extraProfile = {
    opt-level = 1; # TODO bug on 2
  };

  # layers = [
  #   crateUtils.defaultIntermediateLayer
  #   {
  #     crates = [ "sel4-capdl-initializer-core" ];
  #     modifications = seL4Modifications;
  #   }
  # ];

}
