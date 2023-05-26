{ lib
, hostPlatform
, runCommand, runCommandCC
, jq
, symlinkToRegularFile
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

  seL4IncludeDir =
    if worldConfig.isCorePlatform
    then
      let
        d = "${sel4cpForUserspace.sdk}/board/qemu_arm_virt/debug";
      in
        "${d}/include"
    else
      "${seL4ForUserspace}/libsel4/include";

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

  seL4Config =
    let
      f = seg: builtins.fromJSON (builtins.readFile (symlinkToRegularFile "x.json" "${seL4IncludeDir}/${seg}/gen_config.json"));
    in
      f "kernel" // f "sel4";

  seL4ConfigJSON = runCommand "sel4-config.json" {
    nativeBuildInputs = [
      jq
    ];
    json = builtins.toJSON seL4Config;
    passAsFile = [ "json" ];
  } ''
    jq . $jsonPath > $out
  '';

  ###

  mkLoader = { appELF }: mkLoaderWithSerialization {
    app = appELF;
  };

  mkLoaderWithSerialization = callPackage ./mk-loader-with-serialization.nix {};

  mkTask = callPackage ./mk-task.nix {};

  inherit (callPackage ./mk-instance.nix {})
    mkInstance mkCorePlatformInstance mkCapDLRootTask
  ;

  loader-expecting-appended-payload = callPackage ./loader-expecting-appended-payload.nix {
    inherit (worldConfig) loaderConfig;
  };

  capdl-loader = callPackage ./capdl/capdl-loader.nix {};
  objectSizes = callPackage ./capdl/object-sizes.nix {};
  mkSmallCapDLLoader = callPackage ./capdl/mk-capdl-loader.nix {};
  serializeCapDLSpec = callPackage ./capdl/serialize-capdl-spec.nix {};
  dummyCapDLSpec = callPackage ./capdl/dummy-spec.nix {};
  mkSimpleCompositionCapDLSpec = callPackage ./capdl/mk-capdl-spec.nix {};

  mkCapDLLoader = callPackage ./capdl/mk-capdl-loader-with-serialization.nix {};
  # mkCapDLLoader = mkSmallCapDLLoader;

  ###

  docs = callPackage ./docs.nix {};

  ###

  shell = callPackage ./shell.nix {};

  ###

  instances = callPackage ./instances {};
}
