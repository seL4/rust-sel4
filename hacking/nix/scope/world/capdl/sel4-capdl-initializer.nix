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
, mkSeL4RustTargetTriple
}:

mkTask {

  rootCrate = crates.sel4-capdl-initializer;

  targetTriple = mkSeL4RustTargetTriple { minimal = true; };

  release = true;

  # layers = [
  #   crateUtils.defaultIntermediateLayer
  #   {
  #     crates = [ "sel4-capdl-initializer-core" ];
  #     modifications = seL4Modifications;
  #   }
  # ];

}
