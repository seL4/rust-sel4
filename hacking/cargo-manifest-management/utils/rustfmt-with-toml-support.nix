#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ stdenv, lib
, fetchFromGitHub
, rustToolchain
, rustPlatform
}:

let
  # src = fetchFromGitHub {
  #   owner = "xxchan";
  #   repo = "rustfmt";
  #   rev = "1ad83c1d48ac2f5717ea8ae398443510c95734b1";
  #   hash = "sha256-YJ9qNpSnEmOEb45TZcs/HwnZRWOTIXKqvW+f65MtMVE=";
  # };

  src = fetchFromGitHub rec {
    owner = "coliasgroup";
    repo = "rustfmt";
    rev = "742ef5f05bcb527461e685c830c364da0bd46193"; # branch format-cargo-manifest
    hash = "sha256-uZT4qHx6NJaSbQMn2jjxS4wqjWgwi0Zr6qtJzsIH5pc=";
  };

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = "${src}/Cargo.lock";
  };

  cargoConfig = "${cargoDeps}/.cargo/config";

in
stdenv.mkDerivation {
  name = "rustfmt-with-toml-support";

  inherit src;

  nativeBuildInputs = [
    rustToolchain
  ];

  dontConfigure = true;
  dontInstall = true;

  patchPhase = ''
    ln -s ${cargoDeps} cargo-vendor-dir
    ln -s ${cargoDeps}/.cargo .
  '';

  buildPhase = ''
    cargo build \
      -Z unstable-options \
      --offline \
      --frozen \
      --release \
      --out-dir $out/bin \
      -j $NIX_BUILD_CORES
  '';
}
