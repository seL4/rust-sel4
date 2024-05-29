#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, runCommand
, emptyDirectory
, fetchFromGitHub
}:

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

  mkCompilerRTSource = { version, hash }:
    let
      llvmProject = fetchFromGitHub {
        owner = "rust-lang";
        repo = "llvm-project";
        rev = "rustc/${version}";
        inherit hash;
      };
    in
      runCommand "compiler-rt" {} ''
        cp -r ${llvmProject}/compiler-rt $out
      '';

  elaborateRustEnvironment =
    { rustToolchain
    , mkCustomTargetPath ? targetTriple: throw "unimplemented"
    , chooseLinker ? { targetTriple, platform }: null
    , compilerRTSource ? null
    , vendoredSuperLockfile ? null
    }:
    {
      inherit rustToolchain;
      inherit compilerRTSource;
      inherit chooseLinker;
      inherit vendoredSuperLockfile;

      # HACK
      mkTargetPath = targetTriple: if lib.hasInfix "sel4" targetTriple then mkCustomTargetPath targetTriple else emptyDirectory;

      vendoredSysrootLockfile = vendorLockfile {
        inherit rustToolchain;
        lockfile = symlinkToRegularFile "Cargo.lock" "${rustToolchain}/lib/rustlib/src/rust/Cargo.lock";
      };
    };
}
