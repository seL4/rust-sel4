#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, stdenv
, buildPackages
, symlinkJoin

, perl
, dtc

, python312Packages
, llvmPackages_18

, sources
, sdfgen
, microkit

, worldConfig
, callPlatform
}:

let
  llvm = buildPackages.llvmPackages_18;

  # see sddf/flake.nix
  clang = symlinkJoin {
    name = "clang-complete";
    paths = llvm.clang-unwrapped.all;
    postBuild = ''
      cp --remove-destination -- ${llvm.clang-unwrapped}/bin/* $out/bin/
    '';
  };

  inherit (worldConfig) isMicrokit microkitConfig;

  src = sources.sddf + "/examples/serial";

  aggregate = stdenv.mkDerivation {
    name = "x";

    depsBuildBuild = [
      python312Packages.python
      sdfgen
      perl
      dtc
    ];

    nativeBuildInputs = [
      clang
      llvm.lld
      llvm.libllvm
    ];

    MICROKIT_SDK = microkit.sdkWithTool;
    MICROKIT_CONFIG = microkitConfig.config;
    MICROKIT_BOARD = microkitConfig.board;
    MICROKIT_TOOL = "${microkit.tool}/bin/microkit";

    buildCommand = ''
      make -C ${src} BUILD_DIR=$out
    '';
  };

  system = {
    loaderImage = "${aggregate}/loader.img";
  };

in
lib.fix (self: callPlatform {
  inherit system;
} // {
  inherit aggregate;
})
