#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, hostPlatform, buildPackages
, mkShell
, defaultRustToolchain
, pkgconfig, openssl
, cmake, perl, python3Packages
, rustPlatform
}:

mkShell {
  hardeningDisable = [ "all" ];

  nativeBuildInputs = [
    pkgconfig
    rustPlatform.bindgenHook
    defaultRustToolchain
  ];

  buildInputs = [
    openssl
  ];

  depsBuildBuild = [
    buildPackages.stdenv.cc
    cmake
    perl
    python3Packages.jsonschema
    python3Packages.jinja2
  ];
}
