{ runCommand
, capdl-tool
, objectSizes
, mkTask, crates
, crateUtils
, seL4ForUserspace
, seL4RustTargetInfoWithConfig
}:

let
  seL4Modifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      SEL4_PREFIX = seL4ForUserspace;
    });
  };

in mkTask {

  rootCrate = crates.capdl-loader-expecting-serialized-spec;

  rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = true; };

  # release = false;

  extraProfile = {
    opt-level = 1; # bug on 2
  };

  # layers = [
  #   crateUtils.defaultIntermediateLayer
  #   {
  #     crates = [ "capdl-loader-core" ];
  #     modifications = seL4Modifications;
  #   }
  # ];

}
