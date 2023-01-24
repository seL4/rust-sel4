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
    $OBJCOPY -O elf32-i386 ${kernel}/bin/kernel.elf $out
  '';

  seL4ForUserspace = kernel;

  seL4ForBoot = kernel.overrideAttrs (_: {
    # src = lib.cleanSource ../../../../../../../../x/seL4;
  });

  mkLoader = callPackage ./mk-loader.nix {};

  mkTask = callPackage ./mk-task.nix {};

  instances = callPackage ./instances.nix {};
  sel4cpInstances = callPackage ./sel4cp-instances.nix {};

  docs = callPackage ./docs.nix {};

  shell = callPackage ./shell.nix {};
}
