{ runCommand
, capdl-tool
, objectSizes
, mkTask, crates
, serializeCapDLSpec
, crateUtils
, seL4RustEnvVars
, seL4RustTargetInfoWithConfig
}:

{ spec, fill }:

let
  json = serializeCapDLSpec {
    inherit spec;
  };

  seL4Modifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars);
  };

in mkTask {

  rootCrate = crates.capdl-initializer-with-embedded-spec;

  rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = true; };

  extraProfile = {
    opt-level = 1; # bug on 2
  };

  features = [ "deflate" ];

  layers = [
    crateUtils.defaultIntermediateLayer
    {
      crates = [ "capdl-initializer-core" ];
      modifications = seL4Modifications;
    }
  ];

  # release = false;

  lastLayerModifications = crateUtils.composeModifications seL4Modifications (crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      CAPDL_SPEC_FILE = json;
      CAPDL_FILL_DIR = fill;
      CAPDL_OBJECT_NAMES_LEVEL = 2;
      CAPDL_DEFLATE_FILL = 1;
      CAPDL_EMBED_FRAMES = 1;

      passthru = (super.passthru or {}) // {
        inherit spec json fill;
      };
    });
  });

}
