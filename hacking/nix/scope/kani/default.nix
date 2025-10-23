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
, assembleRustToolchain
, parseStructuredChannel
, elaborateRustEnvironment
, mkDefaultElaborateRustEnvironmentArgs
, mkMkCustomTargetPathForEnvironment
}:

let
  rustToolchainAttrs = builtins.fromTOML (builtins.readFile (src + "/rust-toolchain.toml"));

  inherit (rustToolchainAttrs.toolchain) channel;

  rustToolchain = assembleRustToolchain (parseStructuredChannel channel // {
    sha256 = "sha256-6k1KpO4EeeJE65qomJvmJHcwfcK9LlUUaGeQlhA1zbk=";
  });

  rustEnvironment = lib.fix (self: elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
    inherit rustToolchain;
  } // {
    inherit channel;
    mkCustomTargetPath = mkMkCustomTargetPathForEnvironment {
      rustEnvironment = self;
    };
  }));

  src = fetchFromGitHub {
    owner = "model-checking";
    repo = "kani";
    rev = "kani-0.65.0";
    sha256 = "sha256-xle2JCn0HjrWrIkaWbm5mGm0+hPGClMzt3PEO7OgAqg=";
    fetchSubmodules = true;
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
        --prefix PATH : ${lib.makeBinPath [ rustToolchain python3 ]}
    done

    $out/bin/kani setup \
      --use-local-bundle kani-*.tar.gz \
      --use-local-toolchain ${rustToolchain}
  '';

  passthru = {
    inherit rustEnvironment;
  };
}
