{ lib }:

self: with self;

{
  buildCrateInLayers = callPackage ./build-crate-in-layers.nix {};

  buildSysroot = callPackage ./build-sysroot.nix {};

  pruneLockfile = callPackage ./prune-lockfile.nix {};

  vendorLockfile = callPackage ./vendor-lockfile.nix {};

  crateUtils = callPackage ./crate-utils.nix {};

  toTOMLFile = callPackage ./to-toml-file.nix {};

  symlinkToRegularFile = callPackage ./symlink-to-regular-file.nix {};
}
