#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, hostPlatform, buildPackages
, mkShell

, pkg-config
, git
, cacert
, rustup, rustPlatform
, perl
, python3Packages
, cmake

, kani
, verus

, strace
, cntr
, cachix

, openssl

, shellForMakefile
}:

mkShell (shellForMakefile.apply {

  depsBuildBuild = [
    buildPackages.stdenv.cc
  ];

  nativeBuildInputs = [
    pkg-config
    git
    cacert
    rustup
    perl
    cmake
    rustPlatform.bindgenHook
    strace
    cntr
    cachix
  ] ++ lib.optionals hostPlatform.isx86_64 [
    kani
    verus
  ];

  buildInputs = [
    openssl
  ];

  shellHook = ''
    kargo() {
      cargo +${kani.rustEnvironment.channel} "$@"
    }

    vargo() {
      cargo +${verus.rustEnvironment.channel} "$@"
    }
  '';
})
