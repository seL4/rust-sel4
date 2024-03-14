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

  # layers = [
  #   crateUtils.defaultIntermediateLayer
  #   {
  #     crates = [ "sel4-capdl-initializer-core" ];
  #     modifications = seL4Modifications;
  #   }
  # ];

}
