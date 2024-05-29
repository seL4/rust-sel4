#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, hostPlatform, buildPackages
, mkShell

, pkgconfig
, git
, cacert
, rustup, rustPlatform
, perl
, python3Packages
, cmake

, strace
, cntr
, cachix

, openssl

, verus

, shellForMakefile
}:

mkShell (shellForMakefile.apply {

  depsBuildBuild = [
    buildPackages.stdenv.cc
  ];

  nativeBuildInputs = [
    pkgconfig
    git
    cacert
    rustup
    perl
    cmake
    rustPlatform.bindgenHook
    strace
    cntr
    cachix
    verus
  ];

  buildInputs = [
    openssl
  ];
})
