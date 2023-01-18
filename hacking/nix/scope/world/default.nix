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

  seL4ForBoot = kernel;
  seL4ForUserspace = kernel;

  mkLoader = callPackage ./mk-loader.nix {};

  mkTask = callPackage ./mk-task.nix {};

  instances = callPackage ./instances.nix {};
}
