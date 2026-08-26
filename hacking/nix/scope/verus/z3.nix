#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, stdenv
, fetchurl
, autoPatchelfHook
, unzip
}:

let
  version = "4.16.0";

  byArch = {
    "x86_64" = {
      arch = "x64";
      glibcVersion = "2.39";
      sha256 = "sha256-cojEmlvW26/XsLDR9llWuRZy2iSwjwkkKRmvFZvjQY4=";
    };
    "aarch64" = {
      arch = "arm64";
      glibcVersion = "2.38";
      sha256 = lib.fakeHash;
    };
  };

  inherit (byArch.${stdenv.hostPlatform.parsed.cpu.name}) arch glibcVersion sha256;

  filename = "z3-${version}-${arch}-glibc-${glibcVersion}";

in
stdenv.mkDerivation {
  pname = "z3";
  inherit version;

  src = fetchurl {
    url = "https://github.com/Z3Prover/z3/releases/download/z3-${version}/${filename}.zip";
    inherit sha256;
  };

  nativeBuildInputs = [
    stdenv.cc.cc.lib
    autoPatchelfHook
    unzip
  ];

  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    here=$(pwd)
    cd $TMPDIR
    mv $here $out
  '';
}
