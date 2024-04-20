#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, hostPlatform
, runCommand, runCommandCC
, jq
, symlinkToRegularFile
, crateUtils
, mkSeL4
, mkMicrokit
, sources
}:

let
  elaborateWorldConfig =
    { isMicrokit ? false
    , kernelConfig ? null
    , kernelLoaderConfig ? null
    , corePlatformConfig ? null
    , canSimulate ? false
    , platformRequiresLoader ? true
    , mkInstanceForPlatform ? _: { attrs = {}; links = []; }
    }:
    { inherit
        isMicrokit
        kernelConfig
        kernelLoaderConfig
        corePlatformConfig
        canSimulate
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

  microkit = assert worldConfig.isMicrokit; mkMicrokit worldConfig.corePlatformConfig;

  microkitForUserspace = microkit;
  microkitForBoot = microkit;

  seL4 = assert !worldConfig.isMicrokit; mkSeL4 worldConfig.kernelConfig;

  seL4ForUserspace = seL4;

  seL4ForBoot = seL4.overrideAttrs (_: {
    # src = lib.cleanSource (sources.localRoot + "/seL4");
  });

  seL4IncludeDir =
    if worldConfig.isMicrokit
    then
      let
        d = "${microkitForUserspace.sdk}/board/qemu_virt_aarch64/debug";
      in
        "${d}/include"
    else
      "${seL4ForUserspace}/libsel4/include";

  seL4RustEnvVars =
    if worldConfig.isMicrokit
    then
      let
        d = "${microkitForUserspace.sdk}/board/qemu_virt_aarch64/debug";
      in {
          SEL4_INCLUDE_DIRS = "${d}/include";
      }
    else {
      SEL4_PREFIX = seL4ForUserspace;
    };

  seL4Modifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars);
  };

  kernelBinary = assert !worldConfig.isMicrokit; "${seL4ForBoot}/bin/kernel.elf";

  kernelBinary32Bit =
    assert hostPlatform.isx86_64;
    runCommandCC "kernel32.elf" {} ''
      $OBJCOPY -O elf32-i386 ${kernelBinary} $out
    '';

  libsel4 = assert !worldConfig.isMicrokit; "${seL4ForUserspace}/libsel4";

  seL4Config =
    let
      f = seg: builtins.fromJSON (builtins.readFile (symlinkToRegularFile "${seg}-gen_config.json" "${seL4IncludeDir}/${seg}/gen_config.json"));
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

  mkTask = callPackage ./mk-task.nix {};

  sel4-capdl-initializer = callPackage ./capdl/sel4-capdl-initializer.nix {};
  objectSizes = callPackage ./capdl/object-sizes.nix {};
  mkSmallCapDLInitializer = callPackage ./capdl/mk-small-capdl-initializer.nix {};
  serializeCapDLSpec = callPackage ./capdl/serialize-capdl-spec.nix {};
  dummyCapDLSpec = callPackage ./capdl/dummy-capdl-spec.nix {};
  mkSimpleCompositionCapDLSpec = callPackage ./capdl/mk-simple-composition-capdl-spec.nix {};
  mkCapDLInitializerWithSpec = callPackage ./capdl/mk-capdl-initializer-with-spec.nix {};

  # mkCapDLInitializer = mkSmallCapDLInitializer;
  mkCapDLInitializer = mkCapDLInitializerWithSpec;

  sel4-kernel-loader = callPackage ./sel4-kernel-loader.nix {
    inherit (worldConfig) kernelLoaderConfig;
  };

  mkSeL4KernelLoaderWithPayload = { appELF } : callPackage ./mk-sel4-kernel-loader-with-payload.nix {} {
    app = appELF;
  };

  inherit (callPackage ./mk-instance.nix {})
    mkInstance mkMicrokitInstance mkCapDLRootTask
  ;

  instances = callPackage ./instances {};

  docs = callPackage ./docs.nix {};

  shell = callPackage ./shell.nix {};
}
