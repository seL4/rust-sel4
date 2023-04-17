{ lib
, hostPlatform
, runCommandCC
, mkSeL4
, mkSeL4CorePlatform
}:

let
  elaborateWorldConfig =
    { isCorePlatform ? false
    , kernelConfig ? null
    , loaderConfig ? null
    , corePlatformConfig ? null
    , platformRequiresLoader ? true
    , mkInstanceForPlatform ? _: { attrs = {}; links = []; }
    }:
    { inherit
        isCorePlatform
        kernelConfig
        loaderConfig
        corePlatformConfig
        platformRequiresLoader
        mkInstanceForPlatform
      ;
    };
in

unelaboratedWorldConfig:

let
  worldConfig = elaborateWorldConfig unelaboratedWorldConfig;
in

self: with self;

{
  inherit worldConfig;

  sel4cp = assert worldConfig.isCorePlatform; mkSeL4CorePlatform worldConfig.corePlatformConfig;

  sel4cpForUserspace = sel4cp;
  sel4cpForBoot = sel4cp;

  seL4 = assert !worldConfig.isCorePlatform; mkSeL4 worldConfig.kernelConfig;

  seL4ForUserspace = seL4;

  seL4ForBoot = seL4.overrideAttrs (_: {
    # src = lib.cleanSource (sources.localRoot + "/seL4");
  });

  seL4RustEnvVars = 
    if worldConfig.isCorePlatform
    then
      let
        d = "${sel4cpForUserspace.sdk}/board/qemu_arm_virt/debug";
      in {
	      SEL4_INCLUDE_DIRS = "${d}/include";
      }
    else {
      SEL4_PREFIX = seL4ForUserspace;
    };

  kernelBinary = assert !worldConfig.isCorePlatform; "${seL4ForBoot}/bin/kernel.elf";

  kernelBinary32Bit =
    assert hostPlatform.isx86_64;
    runCommandCC "kernel32.elf" {} ''
      $OBJCOPY -O elf32-i386 ${kernelBinary} $out
    '';

  libsel4 = assert !worldConfig.isCorePlatform; "${seL4ForUserspace}/libsel4";

  ###

  mkLoader = callPackage ./mk-loader.nix {
    inherit (worldConfig) loaderConfig;
  };

  mkTask = callPackage ./mk-task.nix {};

  capdl-loader-expecting-serialized-spec = callPackage ./capdl/capdl-loader-expecting-serialized-spec.nix {};
  objectSizes = callPackage ./capdl/object-sizes.nix {};
  mkSmallCapDLLoader = callPackage ./capdl/mk-capdl-loader.nix {};
  mkCapDLLoader = callPackage ./capdl/mk-capdl-loader-with-serialization.nix {};
  serializeCapDLSpec = callPackage ./capdl/serialize-capdl-spec.nix {};
  dummyCapDLSpec = callPackage ./capdl/dummy-spec.nix {};
  mkSimpleCompositionCapDLSpec = callPackage ./capdl/mk-capdl-spec.nix {};

  ###

  docs = callPackage ./docs.nix {};

  ###

  shell = callPackage ./shell.nix {};

  ###

  instances = callPackage ./instances {};
  sel4cpInstances = callPackage ./sel4cp-instances.nix {};
}
