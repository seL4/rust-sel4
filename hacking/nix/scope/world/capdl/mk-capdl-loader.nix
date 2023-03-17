{ runCommand
, capdl-tool
, objectSizes
, mkTask, crates
, serializeCapDLSpec
, crateUtils
, seL4ForUserspace
, seL4RustTargetInfoWithConfig
}:

{ spec, fill }:

let
  json = serializeCapDLSpec {
    inherit spec;
  };

  seL4Modifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      SEL4_PREFIX = seL4ForUserspace;
    });
  };

in mkTask {

  rootCrate = crates.capdl-loader-with-embedded-spec;

  rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = true; };

  # injectPhdrs = true;

  extraProfile = {
    opt-level = 1; # bug on 2
  };

  features = [ "deflate" ];

  layers = [
    crateUtils.defaultIntermediateLayer
    {
      crates = [ "capdl-loader-core" ];
      modifications = seL4Modifications;
    }
  ];

  # release = false;

  lastLayerModifications = crateUtils.composeModifications seL4Modifications (crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      CAPDL_SPEC_FILE = json;
      CAPDL_FILL_DIR = fill;
      CAPDL_OBJECT_NAMES_LEVEL = 2;

      passthru = (super.passthru or {}) // {
        inherit spec json fill;
      };
    });
  });

}
