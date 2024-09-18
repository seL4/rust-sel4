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
, defaultRustEnvironment
, rustEnvironment ? defaultRustEnvironment
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
        --skip-tool \
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
        inherit (rustEnvironment) rustToolchain;
        lockfile = microkitSource + "/tool/microkit/Cargo.lock";
      };

      cargoConfigFile = toTOMLFile "config.toml" vendoredLockfile.configFragment;

    in
      stdenv.mkDerivation ({
        name = "microkit-sdk-just-tool";

        src = lib.cleanSource (microkitSource + "/tool/microkit");

        nativeBuildInputs = [
          rustEnvironment.rustToolchain
        ];

        depsBuildBuild = [
          buildPackages.stdenv.cc
        ];

      } // lib.optionalAttrs (!rustEnvironment.isNightly) {
        # HACK
        RUSTC_BOOTSTRAP = 1;
      } // {

        dontInstall = true;
        dontFixup = true;

        configurePhase = ''
          d=.cargo
          mkdir $d
          cp ${cargoConfigFile} $d/config.toml
        '';

        buildPhase = ''
          cargo build -Z unstable-options --frozen --config ${cargoConfigFile} ${rustEnvironment.artifactDirFlag} $out/bin
        '';
      });

  mkLoader =
    { systemXML
    , searchPath
    }:
    lib.fix (self: runCommand "system" {
      passthru = {
        inherit systemXML;
        image = "${self}/loader.img";
      };
    } ''
      mkdir $out
      MICROKIT_SDK=${sdk} \
        ${tool}/bin/microkit ${systemXML} \
          --search-path ${lib.concatStringsSep " " searchPath} \
          --board ${board} \
          --config ${config} \
          -o $out/loader.img \
          -r $out/report.txt
    '');

  mkSystem =
    { systemXML
    , searchPath
    , extraDebuggingLinks ? []
    , passthru ? {}
    }:
    let
      loader = mkLoader { inherit systemXML searchPath; };
    in {
      inherit loader;
      loaderImage = loader.image;
      debuggingLinks = [
        { name = "loader.img"; path = "${loader}/loader.img"; }
        { name = "report.txt"; path = "${loader}/report.txt"; }
        { name = "sdk/elf"; path = "${sdk}/board/${board}/${config}/elf"; }
        { name = "sel4-symbolize-backtrace";
          path = "${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace";
        }
      ] ++ extraDebuggingLinks;
    } // passthru;

in rec {
  inherit
    sdk tool
    mkSystem
  ;
}
