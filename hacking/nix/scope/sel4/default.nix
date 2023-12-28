#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, writeText, buildPackages
, cmake, ninja
, dtc, libxml2
, python3Packages
, qemuForSeL4
, sources
}:

kernelConfig:

let
  src = sources.seL4.rust;

  settings = writeText "settings.cmake" ''
    ${lib.concatStrings (lib.mapAttrsToList (k: v: ''
      set(${k} ${v.value} CACHE ${v.type} "")
    '') kernelConfig)}
  '';

in
stdenv.mkDerivation {
  name = "seL4";

  inherit src;

  nativeBuildInputs = [
    cmake ninja
    dtc libxml2
    python3Packages.sel4-deps
  ];
  depsBuildBuild = [
    # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
    qemuForSeL4
  ];

  hardeningDisable = [ "all" ];

  postPatch = ''
    # patchShebangs can't handle env -S
    rm configs/*_verified.cmake

    patchShebangs --build .
  '';

  configurePhase = ''
    build=$(pwd)/build

    cmake \
      -DCROSS_COMPILER_PREFIX=${stdenv.cc.targetPrefix} \
      -DCMAKE_INSTALL_PREFIX=$out \
      -C ${settings} \
      -G Ninja \
      -B $build
  '';

  buildPhase = ''
    ninja -C $build all
  '';

  installPhase = ''
    ninja -C $build install
  '';

  dontFixup = true;
}
