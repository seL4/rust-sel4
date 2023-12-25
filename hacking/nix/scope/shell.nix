#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, hostPlatform, buildPackages
, mkShell
, cacert, git
, defaultRustToolchain
, pkgconfig, openssl
, cmake, perl, python3Packages
, rustPlatform
, reuse
, cargo-audit
, strace
, cntr
, cachix
}:

mkShell {
  hardeningDisable = [ "all" ];

  depsBuildBuild = [
    buildPackages.stdenv.cc
    cacert
    git
    cmake
    perl
    python3Packages.jsonschema
    python3Packages.jinja2
    reuse
    cargo-audit
    strace
    cntr
    cachix
  ];

  nativeBuildInputs = [
    pkgconfig
    rustPlatform.bindgenHook
    defaultRustToolchain
  ];

  buildInputs = [
    openssl
  ];
}
