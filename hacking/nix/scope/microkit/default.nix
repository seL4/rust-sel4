#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPlatform, hostPlatform
, buildPackages, pkgsBuildBuild
, linkFarm, writeScript, runCommand
, buildEnv
, callPackage
, cmake, ninja
, dtc, libxml2
, python312Packages
, fenix
, libclangPath
, qemuForSeL4
, sources
, toTOMLFile
}:

worldConfig:

let
  inherit (worldConfig) platformRequiresLoader microkitConfig;
  inherit (microkitConfig) board config;

  microkitSource = sources.microkit;

  kernelSource = sources.seL4;

  kernelSourcePatched = stdenv.mkDerivation {
    name = "kernel-source-for-microkit";
    src = kernelSource;
    phases = [ "unpackPhase" "patchPhase" "installPhase" ];
    nativeBuildInputs = [
      python312Packages.sel4-deps
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

  sdkArch =
    if hostPlatform.isAarch64 then "aarch64"
    else if hostPlatform.isRiscV64 then "riscv64"
    else if hostPlatform.isx86_64 then "x86_64"
    else throw "unknown arch";

  rustToolchain = fenix.fromToolchainFile {
    file = ./rust-toolchain.toml;
    sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
  };

  sdk =
    let
      inherit (pkgsBuildBuild.rustPlatform) importCargoLock;

      vendoredWorkspace = importCargoLock {
        lockFile = microkitSource + "/Cargo.lock";
        allowBuiltinFetchGit = true;
      };

      vendoredStdWorkspace = importCargoLock {
        lockFile = rustToolchain + "/lib/rustlib/src/rust/library/Cargo.lock";
        allowBuiltinFetchGit = true;
      };

      vendored = runCommand "vendored" {
        nativeBuildInputs = [
          python312Packages.toml
        ];
      } ''
        mkdir $out
        for d in ${vendoredStdWorkspace}/* ${vendoredWorkspace}/*; do
          name=$(basename $d)
          if [ -d $out/$name ]; then
            ! diff -r -q $d $out/$name
          else
            cp -r $d $out
          fi
        done

        mkdir $out/.cargo
        python3 ${./merge-config.py} \
          -d $out \
          ${vendoredStdWorkspace}/.cargo/config.toml \
          ${vendoredWorkspace}/.cargo/config.toml \
             > $out/.cargo/config.toml
      '';
    in
      stdenv.mkDerivation {
        name = "microkit-sdk";

        src = microkitSource;

        LIBCLANG_PATH = libclangPath;

        nativeBuildInputs = [
          cmake ninja
          dtc libxml2
          python312Packages.sel4-deps
          rustToolchain
        ];

        depsBuildBuild = [
          buildPackages.stdenv.cc
          # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
          qemuForSeL4
        ];

        dontFixup = true;

        configurePhase = ''
          cat ${vendored}/.cargo/config.toml >> .cargo/config.toml
        '';

        buildPhase = ''
          python3 build_sdk.py \
            --sel4 ${kernelSourcePatched} \
            --boards ${board} \
            --configs ${config} \
            --gcc-toolchain-prefix-${sdkArch} ${lib.removeSuffix "-" stdenv.cc.targetPrefix} \
            --skip-docs \
            --skip-tar
        '';

        installPhase = ''
          mv release/microkit-sdk-* $out
        '';
      };

  imageName = if platformRequiresLoader then "loader" else "root-task";

  mkSystemImage =
    { systemXML
    , searchPath
    }:
    lib.fix (self: runCommand "system" {
      passthru = {
        inherit systemXML;
        image = "${self}/${imageName}.img";
      };
    } ''
      mkdir $out
      MICROKIT_SDK=${sdk} \
        ${sdk}/bin/microkit ${systemXML} \
          --search-path ${lib.concatStringsSep " " searchPath} \
          --board ${board} \
          --config ${config} \
          -o $out/${imageName}.img \
          -r $out/report.txt
    '');

  mkSystem =
    { systemXML
    , searchPath
    , extraDebuggingLinks ? []
    , passthru ? {}
    }:
    let
      system = mkSystemImage { inherit systemXML searchPath; };
    in {
      inherit system;
      "${if platformRequiresLoader then "loaderImage" else "rootTaskImage"}" = system.image;
      debuggingLinks = [
        { name = "${imageName}.img"; path = system.image; }
        { name = "report.txt"; path = "${system}/report.txt"; }
        { name = "sdk/elf"; path = "${sdk}/board/${board}/${config}/elf"; }
        { name = "sel4-symbolize-backtrace";
          path = "${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace";
        }
      ] ++ extraDebuggingLinks;
    } // passthru;

in rec {
  inherit
    sdk
    mkSystem
  ;
}
