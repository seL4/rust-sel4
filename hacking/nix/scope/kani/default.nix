#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, callPackage
, fetchFromGitHub
, makeWrapper
, python3
, python3Packages
, cbmc
, kissat

, crateUtils
, vendorLockfile
, sources
, fenix
, elaborateRustEnvironment
, mkDefaultElaborateRustEnvironmentArgs
, mkMkCustomTargetPathForEnvironment
}:

let
  cbmc-viewer = python3Packages.callPackage ./cbmc-viewer.nix {};

  rustToolchainAttrs = builtins.fromTOML (builtins.readFile (src + "/rust-toolchain.toml"));

  rustToolchain = fenix.fromToolchainFile {
    file = crateUtils.toTOMLFile "rust-toolchain.toml" (crateUtils.clobber [
      rustToolchainAttrs
      {
        toolchain.components = rustToolchainAttrs.toolchain.components ++ [ "rust-src" ];
      }
    ]);
    sha256 = "sha256-akRjdj8CP0mRL4czKC7GssksdunxdckSNiNU36zduV0=";
  };

  rustEnvironment = lib.fix (self: elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
    inherit rustToolchain;
  } // {
    inherit (rustToolchainAttrs.toolchain) channel;
    mkCustomTargetPath = mkMkCustomTargetPathForEnvironment {
      rustEnvironment = self;
    };
  }));

  src = fetchFromGitHub {
    owner = "model-checking";
    repo = "kani";
    rev = "kani-0.52.0";
    sha256 = "sha256-LyMZt6E1U03n4w5L9JMxgklAyvhltSJTvRl7zxYpw6A=";
  };

  localLockfile = vendorLockfile {
    inherit rustToolchain;
    lockfile = src + "/Cargo.lock";
  };

  sysrootLockfile = rustEnvironment.vendoredSysrootLockfile;

  augmentedLockfileValue = {
    version =
      assert localLockfile.lockfileValue.version == sysrootLockfile.lockfileValue.version;
      localLockfile.lockfileValue.version;
    package = localLockfile.lockfileValue.package ++ sysrootLockfile.lockfileValue.package;
  };

  augmentedLockfile = vendorLockfile {
    inherit rustToolchain;
    lockfileValue = augmentedLockfileValue;
  };

  configFragment = crateUtils.toTOMLFile "config" augmentedLockfile.configFragment;

  pythonEnv = python3.buildEnv.override {
    extraLibs = [ cbmc-viewer ];
  };

in
stdenv.mkDerivation {
  name = "kani";

  inherit src;

  nativeBuildInputs = [
    rustToolchain
    makeWrapper
    cbmc
    kissat
  ];

  postPatch = ''
    # HACK
    substituteInPlace src/setup.rs \
      --replace \
        'Command::new("python3")' \
        'Command::new("true")'
  '';

  configurePhase = ''
    cat ${configFragment} >> .cargo/config.toml
  '';

  buildPhase = ''
    RUSTUP_HOME=/var/empty/rustup \
    RUSTUP_TOOLCHAIN=none \
      cargo bundle
  '';

  installPhase = ''
    install -D -t $out/bin target/kani/bin/{kani,cargo-kani}

    for p in $out/bin/*; do
      wrapProgram $p \
        --set KANI_HOME $out/lib/kani \
        --prefix PATH : ${lib.makeBinPath [ rustToolchain pythonEnv ]}
    done

    $out/bin/kani setup \
      --use-local-bundle kani-*.tar.gz \
      --use-local-toolchain ${rustToolchain}
  '';

  passthru = {
    inherit rustEnvironment;
    inherit cbmc-viewer;
  };
}
