#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPlatform
, buildPackages, pkgsBuildBuild
, linkFarm, writeScript, runCommand
, callPackage
, cmake, ninja
, dtc, libxml2
, python3Packages
, qemuForSeL4
, sources
, vendorLockfile
, toTOMLFile
, defaultRustToolchain
, rustToolchain ? defaultRustToolchain
}:

# no configuration yet
{}:

let
  microkitSource = sources.microkit;

  kernelSource = sources.seL4.rust-microkit;

  kernelSourcePatched = stdenv.mkDerivation {
    name = "kernel-source-for-microkit";
    src = kernelSource;
    phases = [ "unpackPhase" "patchPhase" "installPhase" ];
    nativeBuildInputs = [
      python3Packages.sel4-deps
    ];
    postPatch = ''
      # patchShebangs can't handle env -S
      rm configs/*_verified.cmake

      patchShebangs --build .
    '';
    installPhase = ''
      cp -R ./ $out
    '';
  };

  vendoredLockfile = vendorLockfile {
    inherit rustToolchain;
    lockfile = microkitSource + "/tool/microkit/Cargo.lock";
  };

  cargoConfigFile = toTOMLFile "config.toml" vendoredLockfile.configFragment;

  sdk = stdenv.mkDerivation {
    name = "microkit-sdk";

    src = microkitSource;

    nativeBuildInputs = [
      cmake ninja
      dtc libxml2
      python3Packages.sel4-deps
      rustToolchain
    ];

    depsBuildBuild = [
      buildPackages.stdenv.cc
      # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
      qemuForSeL4
    ];

    dontFixup = true;

    configurePhase = ''
      d=tool/microkit/.cargo
      mkdir $d
      cp ${cargoConfigFile} $d/config.toml
    '';

    buildPhase = ''
      python3 build_sdk.py \
        --sel4=${kernelSourcePatched} \
        --tool-target-triple=${buildPlatform.config}
    '';

    installPhase = ''
      mv release/microkit-sdk-* $out
    '';
  };

  exampleSource = microkitSource + "/example/qemu_virt_aarch64/hello";

  examplePDs = stdenv.mkDerivation {
    name = "example";

    src = exampleSource;

    MICROKIT_SDK = sdk;
    MICROKIT_BOARD = "qemu_virt_aarch64";
    MICROKIT_CONFIG = "debug";

    MICROKIT_TOOL = "${sdk}/bin/microkit";

    dontConfigure = true;
    dontFixup = true;

    buildPhase = ''
      mkdir build
      make BUILD_DIR=build
    '';

    installPhase = ''
      mkdir $out
      mv build/hello.elf $out
    '';
  };

  mkSystem = { searchPath, systemXML }:
    lib.fix (self: runCommand "system" {
      MICROKIT_SDK = sdk;
      MICROKIT_BOARD = "qemu_virt_aarch64";
      MICROKIT_CONFIG = "debug";

      nativeBuildInputs = [
        python3Packages.sel4-deps
      ];

      passthru = rec {
        loader = "${self}/loader.img";
        links = [
          { name = "pds"; path = searchPath; }
          { name = "loader.elf"; path = loader; }
          { name = "report.txt"; path = "${self}/report.txt"; }
          { name = "sdk/monitor.elf"; path = "${sdk}/board/qemu_virt_aarch64/debug/elf/monitor.elf"; }
          { name = "sdk/loader.elf"; path = "${sdk}/board/qemu_virt_aarch64/debug/elf/loader.elf"; }
        ];
      };
    } ''
      mkdir $out
      ${sdk}/bin/microkit ${systemXML} \
        --search-path ${searchPath} \
        --board $MICROKIT_BOARD \
        --config $MICROKIT_CONFIG \
        -o $out/loader.img \
        -r $out/report.txt
    '');

  example = mkSystem {
    searchPath = examplePDs;
    systemXML = exampleSource + "/hello.system";
  };

in rec {
  inherit
    sdk
    mkSystem
    example
  ;
}
