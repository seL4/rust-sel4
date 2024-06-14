#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ stdenv, hostPlatform
, fetchurl
, autoPatchelfHook
, unzip
}:

let
  version = "4.12.5";

  byArch = {
    "x86_64" = {
      arch = "x64";
      sha256 = "sha256-8DZXTV4gKckgT/81A8/mjd9B+m/euzm+7ZnhvzVbf+4=";
    };
    "aarch64" = {
      arch = "arm64";
      sha256 = "sha256-FeX6pi5lvRitHAUbGqbxdqqyjeEU0yRsaQsj8vyj12k=";
    };
  };

  inherit (byArch.${hostPlatform.parsed.cpu.name}) arch sha256;

  filename = "z3-${version}-${arch}-glibc-2.35";

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
