{ lib
, hostPlatform
, runCommandCC
}:

{ kernelConfig
, loaderConfig
, ...
} @ worldConfig:

self: with self;

{
  inherit worldConfig;
  inherit kernelConfig loaderConfig mkQemuCmd;

  kernel = callPackage ./kernel.nix {};

  kernel32Bit = assert hostPlatform.isx86_64; runCommandCC "kernel32.elf" {} ''
    $OBJCOPY -O elf32-i386 ${seL4ForBoot}/bin/kernel.elf $out
  '';

  seL4ForUserspace = kernel;

  seL4ForBoot = kernel.overrideAttrs (_: {
    # src = lib.cleanSource ../../../../../../../../x/seL4;
  });

  libsel4 = "${seL4ForUserspace}/libsel4";

  mkLoader = callPackage ./mk-loader.nix {};

  mkTask = callPackage ./mk-task.nix {};

  instances = callPackage ./instances {};
  sel4cpInstances = callPackage ./sel4cp-instances.nix {};

  docs = callPackage ./docs.nix {};

  shell = callPackage ./shell.nix {};

  serializeCapDLSpec = callPackage ./capdl/serialize-capdl-spec.nix {};
  mkCapDLSpec = callPackage ./capdl/mk-capdl-spec.nix {};
  mkSmallCapDLLoader = callPackage ./capdl/mk-capdl-loader.nix {};
  dummyCapDLSpec = callPackage ./capdl/dummy-spec.nix {};
  objectSizes = callPackage ./capdl/object-sizes.nix {};
  capdl-loader-expecting-serialized-spec = callPackage ./capdl/capdl-loader-expecting-serialized-spec.nix {};
  mkCapDLLoader = callPackage ./capdl/mk-capdl-loader-with-serialization.nix {};
}
