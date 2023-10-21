#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ stdenv
, fetchFromGitHub
, rustToolchain
, crateUtils
, vendorLockfile
}:

let
  src = fetchFromGitHub {
    owner = "xxchan";
    repo = "rustfmt";
    rev = "1ad83c1d48ac2f5717ea8ae398443510c95734b1";
    hash = "sha256-YJ9qNpSnEmOEb45TZcs/HwnZRWOTIXKqvW+f65MtMVE=";
  };

  cargoConfig = crateUtils.toTOMLFile "config" (crateUtils.clobber [
    {
      unstable.unstable-options = true;
    }
    (vendorLockfile { lockfile = "${src}/Cargo.lock"; }).configFragment
  ]);

in
stdenv.mkDerivation {
  name = "rustfmt-with-toml-support";

  inherit src;

  nativeBuildInputs = [
    rustToolchain
  ];

  dontConfigure = true;
  dontInstall = true;

  buildPhase = ''
    cargo build \
      --config ${cargoConfig} \
      --offline \
      --frozen \
      --release \
      --out-dir $out/bin \
      -j $NIX_BUILD_CORES
  '';
}
