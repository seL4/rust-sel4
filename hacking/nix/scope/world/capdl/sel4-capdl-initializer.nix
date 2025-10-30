#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, runCommand
, capdl-tool
, objectSizes
, mkTask, crates
, crateUtils
, seL4Modifications
, mkSeL4RustTargetTriple

, deflate ? true
, alloc ? true
}:

mkTask {

  rootCrate = crates.sel4-capdl-initializer;

  targetTriple = mkSeL4RustTargetTriple { minimal = true; };

  release = true;

  noDefaultFeatures = true;
  features = lib.optional deflate "deflate" ++ lib.optional deflate "alloc";

  # layers = [
  #   crateUtils.defaultIntermediateLayer
  #   {
  #     crates = [ "sel4-capdl-initializer-core" ];
  #     modifications = seL4Modifications;
  #   }
  # ];

}
