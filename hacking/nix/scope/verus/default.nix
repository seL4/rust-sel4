#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, callPackage
, makeWrapper
, singular

, crateUtils
, vendorLockfile
, sources
, assembleRustToolchain
, elaborateRustEnvironment
, mkDefaultElaborateRustEnvironmentArgs
, mkMkCustomTargetPathForEnvironment
}:

let
  z3 = callPackage ./z3.nix {};

  rustToolchainAttrs = builtins.fromTOML (builtins.readFile (src + "/rust-toolchain.toml"));

  rustToolchain = assembleRustToolchain {
    channel = "1.76.0";
    sha256 = "sha256-e4mlaJehWBymYxJGgnbuCObVlqMlQSilZ8FljG9zPHY=";
  };

  rustEnvironment = lib.fix (self: elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
    inherit rustToolchain;
  } // {
    inherit (rustToolchainAttrs.toolchain) channel;
    backwardsCompatibilityHacks = {
      outDirInsteadOfArtifactDir = true;
      noLibraryWorkspace = true;
    };
    mkCustomTargetPath = mkMkCustomTargetPathForEnvironment {
      rustEnvironment = self;
    };
  }));

  src = sources.fetchGit {
    url = "https://github.com/coliasgroup/verus.git";
    rev = "c1d8b986315b1d7fcaa0bf63c2e0497fbebab231"; # branch dev
  };

  lockfile = vendorLockfile {
    inherit rustToolchain;
    lockfile = src + "/source/Cargo.lock";
  };

  config = crateUtils.toTOMLFile "config" lockfile.configFragment;

in
stdenv.mkDerivation {
  name = "verus";

  inherit src;

  sourceRoot = "source/source"; # looks funny

  nativeBuildInputs = [
    rustToolchain
    makeWrapper
  ];

  dontConfigure = true;

  buildPhase = ''
    RUSTC_BOOTSTRAP=1 \
      cargo build \
        -Z unstable-options \
        --config ${config} \
        -p verus-driver -p cargo-verus \
        --features=verus-driver/singular \
        --release \
        ${rustEnvironment.artifactDirFlag} $out/bin
  '';

  installPhase = ''
    # wrap cargo-verus with these so that it can properly assess whether verus-driver fingerprints are dirty
    wrapProgram $out/bin/cargo-verus \
      --set-default VERUS_Z3_PATH ${z3}/bin/z3 \
      --set-default VERUS_SINGULAR_PATH ${singular}/bin/Singular

    wrapProgram $out/bin/verus-driver \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ rustToolchain ]} # HACK
  '';

  passthru = {
    inherit rustEnvironment;
    inherit z3;
  };
}
