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

  inherit (rustToolchainAttrs.toolchain) channel;

  rustToolchain = assembleRustToolchain {
    inherit channel;
    sha256 = "sha256-yMuSb5eQPO/bHv+Bcf/US8LVMbf/G/0MSfiPwBhiPpk=";
  };

  rustEnvironment = lib.fix (self: elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
    inherit rustToolchain;
  } // {
    inherit channel;
    mkCustomTargetPath = mkMkCustomTargetPathForEnvironment {
      rustEnvironment = self;
    };
  }));

  src = sources.fetchGit {
    url = "https://github.com/coliasgroup/verus.git";
    rev = "85249163e18625abaf1e0822c81d0839cc348ac4"; # branch dev
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
