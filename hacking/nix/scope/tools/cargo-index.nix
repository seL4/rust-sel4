#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, rustPlatform, fetchFromGitHub
, pkgconfig, openssl
, sources
}:

rustPlatform.buildRustPackage rec {
  name = "cargo-index";

  src = fetchFromGitHub {
    owner = "nspin";
    repo = "cargo-index";
    rev = "a0cc73ab5d2b56d411893c62ec748a945d11d22a";
    hash = "sha256-B734Svj/dlNiDd0o4apbwbjzsRPcrAGNFVICdHQdGiw=";
  };

  # src = sources.localRoot + "/cargo-index";

  cargoSha256 = "sha256-1KvEq+hTJEBKPeaSOe/m41/VCtG0skyQX4TuSvHo1HI=";

  nativeBuildInputs = [
    pkgconfig
  ];

  buildInputs = [
    openssl
  ];
}
