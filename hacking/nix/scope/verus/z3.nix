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

  arch = {
    "x86_64" = "x64";
    "aarch64" = "arm64";
  }.${hostPlatform.parsed.cpu.name};

  filename = "z3-${version}-${arch}-glibc-2.35";

in
stdenv.mkDerivation {
  pname = "z3";
  inherit version;

  src = fetchurl {
    url = "https://github.com/Z3Prover/z3/releases/download/z3-${version}/${filename}.zip";
    sha256 = "sha256-8DZXTV4gKckgT/81A8/mjd9B+m/euzm+7ZnhvzVbf+4=";
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
