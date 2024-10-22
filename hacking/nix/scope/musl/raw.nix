#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, fetchFromGitHub }:

stdenv.mkDerivation rec {
  name = "musl";

  src = fetchFromGitHub {
    owner = "seL4";
    repo = "musllibc";
    rev = "9798aedbc3ee5fa3c1d7f788e9312df9203e7b0b";
    hash = "sha256-EZUdUIXW/zhHKxconwYm0gkSX70/+U8JfldTJmYRtgY=";
  };

  hardeningDisable = [ "all" ]; # TODO

  NIX_CFLAGS_COMPILE = [
    "-fdebug-prefix-map=.=${src}"
  ];

  dontDisableStatic = true;
  dontFixup = true;

  configureFlags = [
    "--enable-debug"
    "--enable-warnings"
    "--disable-shared"
    "--enable-static"
    "--disable-optimize"
    "--disable-visibility" # HACK
  ];

  postConfigure = ''
    sed -i 's/^ARCH = \(.*\)/ARCH = \1_sel4/' config.mak
  '';

  makeFlags = [
    "-f" "Makefile.muslc"
  ];
}
