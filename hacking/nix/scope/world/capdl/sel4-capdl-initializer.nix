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

  # release = false;

  # extraProfile = {
  #   opt-level = 1; # bug on 2
  # };

  # layers = [
  #   crateUtils.defaultIntermediateLayer
  #   {
  #     crates = [ "sel4-capdl-initializer-core" ];
  #     modifications = seL4Modifications;
  #   }
  # ];

}
