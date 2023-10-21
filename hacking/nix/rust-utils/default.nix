#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib }:

self: with self;

let
  # HACK: unify across cross pkgsets
  callBuildBuildPackage = otherSplices.selfBuildBuild.callPackage;
in

{
  buildCrateInLayers = callPackage ./build-crate-in-layers.nix {};

  buildSysroot = callPackage ./build-sysroot.nix {};

  pruneLockfile = callBuildBuildPackage ./prune-lockfile.nix {};

  vendorLockfile = callBuildBuildPackage ./vendor-lockfile.nix {};

  crateUtils = callPackage ./crate-utils.nix {};

  toTOMLFile = callBuildBuildPackage ./to-toml-file.nix {};

  symlinkToRegularFile = callBuildBuildPackage ./symlink-to-regular-file.nix {};
}
