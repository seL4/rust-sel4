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
, fenix
}:

let
  z3 = callPackage ./z3.nix {};

  rustToolchainAttrs = builtins.fromTOML (builtins.readFile (src + "/rust-toolchain.toml"));

  rustToolchain = fenix.fromToolchainFile {
    file = crateUtils.toTOMLFile "rust-toolchain.toml" (crateUtils.clobber [
      rustToolchainAttrs
      {
        toolchain.components = rustToolchainAttrs.toolchain.components ++ [ "rust-src" ];
      }
    ]);
    sha256 = "sha256-e4mlaJehWBymYxJGgnbuCObVlqMlQSilZ8FljG9zPHY=";
  };

  src = sources.fetchGit {
    url = "https://github.com/nspin/verus.git";
    rev = "fdac1c3c52e639bf3e835802f63b43520379b1a1";
    local = sources.localRoot + "/../s/verus-hacking/verus";
    # useLocal = true;
  };

  lockfile = vendorLockfile {
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
        --out-dir $out/bin
  '';

  installPhase = ''
    # wrap cargo-verus with these so that it can properly assess whether verus-driver fingerprints are dirty
    wrapProgram $out/bin/cargo-verus \
      --set-default VERUS_Z3_PATH ${z3}/bin/z3 \
      --set-default VERUS_SINGULAR_PATH ${singular}/bin/Singular

    wrapProgram $out/bin/verus-driver \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ rustToolchain ]} # HACK
  '';
}
