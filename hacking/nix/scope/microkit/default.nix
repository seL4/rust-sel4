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

{ board, config }:

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

  sdk = stdenv.mkDerivation {
    name = "microkit-sdk-without-tool";

    src = lib.cleanSourceWith {
      src = microkitSource;
      filter = name: type:
        let baseName = baseNameOf (toString name);
        in !(type == "directory" && baseName == "tool");
    };

    nativeBuildInputs = [
      cmake ninja
      dtc libxml2
      python3Packages.sel4-deps
    ];

    depsBuildBuild = [
      # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
      qemuForSeL4
    ];

    dontConfigure = true;
    dontFixup = true;

    buildPhase = ''
      python3 build_sdk.py \
        --sel4=${kernelSourcePatched} \
        --boards ${board} \
        --configs ${config} \
        --skip-docs \
        --skip-tar

    '';

    installPhase = ''
      mv release/microkit-sdk-* $out
    '';
  };

  tool =
    let
      vendoredLockfile = vendorLockfile {
        inherit rustToolchain;
        lockfile = microkitSource + "/tool/microkit/Cargo.lock";
      };

      cargoConfigFile = toTOMLFile "config.toml" vendoredLockfile.configFragment;

    in
      stdenv.mkDerivation {
        name = "microkit-sdk-just-tool";

        src = lib.cleanSource (microkitSource + "/tool/microkit");

        nativeBuildInputs = [
          rustToolchain
        ];

        depsBuildBuild = [
          buildPackages.stdenv.cc
        ];

        dontInstall = true;
        dontFixup = true;

        configurePhase = ''
          d=.cargo
          mkdir $d
          cp ${cargoConfigFile} $d/config.toml
        '';

        buildPhase = ''
          cargo build -Z unstable-options --frozen --config ${cargoConfigFile} --out-dir $out/bin
        '';
      };

  mkSystem = { searchPath, systemXML }:
    lib.fix (self: runCommand "system" {
      passthru = rec {
        inherit systemXML;
        loader = "${self}/loader.img";
        links = [
          { name = "pds"; path = searchPath; }
          { name = "loader.elf"; path = loader; }
          { name = "report.txt"; path = "${self}/report.txt"; }
          { name = "sdk/monitor.elf"; path = "${sdk}/board/${board}/${config}/elf/monitor.elf"; }
          { name = "sdk/loader.elf"; path = "${sdk}/board/${board}/${config}/elf/loader.elf"; }
        ];
      };
    } ''
      mkdir $out
      MICROKIT_SDK=${sdk} \
        ${tool}/bin/microkit ${systemXML} \
          --search-path ${searchPath} \
          --board ${board} \
          --config ${config} \
          -o $out/loader.img \
          -r $out/report.txt
    '');

in rec {
  inherit
    sdk tool
    mkSystem
  ;
}
