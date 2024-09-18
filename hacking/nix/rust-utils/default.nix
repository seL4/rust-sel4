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
  buildCratesInLayers = callPackage ./build-crates-in-layers.nix {};

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

  mkBuiltinRustTargetTriple = name: {
    inherit name;
    isBuiltin = true;
  };

  mkCustomRustTargetTriple = name: {
    inherit name;
    isBuiltin = false;
  };

  elaborateRustEnvironment = lib.makeOverridable (
    { rustToolchain
    , channel ? null
    , isNightly ?
        if channel != null
        then lib.hasPrefix "nightly" channel
        else throw "could not determine isNightly automatically"
    , backwardsCompatibilityHacks ? {}
    , mkCustomTargetPath ? customTargetTripleTripleName: throw "unimplemented"
    , chooseLinker ? { targetTriple, platform }: null
    , compilerRTSource ? null
    , vendoredSuperLockfile ? null
    }:
    let
      elaborateBackwardsCompatibilityHacks =
        { outDirInsteadOfArtifactDir ? false
        , noLibraryWorkspace ? false
        }:
        {
          inherit
            outDirInsteadOfArtifactDir
            noLibraryWorkspace
          ;
        };
      elaboratedBackwardsCompatibilityHacks = elaborateBackwardsCompatibilityHacks backwardsCompatibilityHacks;
    in {
      inherit rustToolchain channel isNightly;
      inherit compilerRTSource;
      inherit chooseLinker;
      inherit vendoredSuperLockfile;

      artifactDirFlag = if elaboratedBackwardsCompatibilityHacks.outDirInsteadOfArtifactDir then "--out-dir" else "--artifact-dir";

      mkTargetPath = targetTriple: if !targetTriple.isBuiltin then mkCustomTargetPath targetTriple.name else emptyDirectory;

      vendoredSysrootLockfile = vendorLockfile {
        inherit rustToolchain;
        lockfile = symlinkToRegularFile "Cargo.lock" "${rustToolchain}/lib/rustlib/src/rust/${lib.optionalString (!elaboratedBackwardsCompatibilityHacks.noLibraryWorkspace) "library/"}Cargo.lock";
      };
    }
  );
}
