#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPlatform
, callPackage
, makeWrapper
, writeShellScriptBin
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
    sha256 = "sha256-Hn2uaQzRLidAWpfmRwSRdImifGUCAb9HeAqTYFXWeQk=";
  };

  rustEnvironment = lib.fix (self: elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
    inherit rustToolchain;
  } // {
    inherit channel;
    mkCustomTargetPath = mkMkCustomTargetPathForEnvironment {
      rustEnvironment = self;
    };
  }));

  toolchainName = "${rustEnvironment.channel}-${buildPlatform.config}";

  src = sources.fetchGit {
    url = "https://github.com/verus-lang/verus.git";
    rev = "91be9dfa7609463973107093ed97f8ad1640d9ed";
  };

  lockfile = vendorLockfile {
    inherit rustToolchain;
    lockfile = src + "/source/Cargo.lock";
  };

  vargoLockfile = vendorLockfile {
    inherit rustToolchain;
    lockfile = src + "/tools/vargo/Cargo.lock";
  };

  config = crateUtils.toTOMLFile "config" lockfile.configFragment;

  vargoConfig = crateUtils.toTOMLFile "config" vargoLockfile.configFragment;

  buildtimeDummyRustup = writeShellScriptBin "rustup" ''
    set -eu -o pipefail

    args="$@"

    die() {
      echo "unexpected args: $args" >&2
      exit 1
    }

    [ "$1" == "show" ] || die
    shift

    [ "$1" == "active-toolchain" ] || die
    shift

    echo "${toolchainName}"
  '';

  runtimeDummyRustup = writeShellScriptBin "rustup" ''
    set -eu -o pipefail

    args="$@"

    die() {
      echo "unexpected args: $args" >&2
      exit 1
    }

    case "$1" in
      run)
          [ "$1" == "run" ] || die
          shift
          shift
          [ "$1" == "--" ] || die
          shift
          exec "$@"
        ;;
      toolchain)
          [ "$1" == "toolchain" ] || die
          shift
          [ "$1" == "list" ] || die
          shift
          echo "${toolchainName}"
        ;;
      *)
        die
        ;;
    esac
  '';

in
stdenv.mkDerivation {
  name = "verus";

  inherit src;

  nativeBuildInputs = [
    rustToolchain
    makeWrapper
    buildtimeDummyRustup
  ];

  VERUS_Z3_PATH = "${z3}/bin/z3";
  VERUS_SINGULAR_PATH = "${singular}/bin/Singular";

  patchPhase = ''
    substituteInPlace tools/activate --replace-fail 'cargo build' 'cargo build --offline'
  '';

  configurePhase = ''
    cat ${vargoConfig} >> tools/vargo/.cargo/config.toml
    mkdir source/.cargo
    cp ${config} source/.cargo/config.toml
  '';

  buildPhase = ''
    cd source
    source ../tools/activate
    vargo build --release
  '';

  installPhase = ''
    mkdir -p $out/lib $out/bin
    cp -r target-verus/release $out/lib/verus-root
    ln -s ../lib/verus-root/cargo-verus $out/bin

    wrapProgram $out/lib/verus-root/verus \
      --prefix PATH : ${
        lib.makeBinPath [
          runtimeDummyRustup
        ]
      } \
      --prefix LD_LIBRARY_PATH : ${rustToolchain}/lib/rustlib/x86_64-unknown-linux-gnu/lib
  '';

  passthru = {
    inherit rustEnvironment;
    inherit z3;
  };
}
