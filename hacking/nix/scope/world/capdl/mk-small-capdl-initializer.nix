#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand
, capdl-tool
, objectSizes
, mkTask, crates
, serializeCapDLSpec
, crateUtils
, seL4Modifications
, mkSeL4RustTargetTriple
}:

{ cdl, fill }:

let
  json = serializeCapDLSpec {
    inherit cdl;
  };

in mkTask {

  rootCrate = crates.sel4-capdl-initializer-with-embedded-spec;

  targetTriple = mkSeL4RustTargetTriple { minimal = true; };

  release = true;

  features = [ "deflate" ];

  layers = [
    crateUtils.defaultIntermediateLayer
    {
      crates = [ "sel4-capdl-initializer-core" ];
      modifications = seL4Modifications;
    }
  ];

  lastLayerModifications = crateUtils.composeModifications seL4Modifications (crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      CAPDL_SPEC_FILE = json;
      CAPDL_FILL_DIR = fill;
      CAPDL_OBJECT_NAMES_LEVEL = 2;
      # CAPDL_DEFLATE_FILL = 1; # TODO broken
      CAPDL_EMBED_FRAMES = 1;

      passthru = (super.passthru or {}) // {
        inherit cdl json fill;
      };
    });
  });

}
