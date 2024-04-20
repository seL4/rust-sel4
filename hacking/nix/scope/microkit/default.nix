#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, writeScript, runCommand
, callPackage
, cmake, ninja
, dtc, libxml2
, python3Packages
, qemuForSeL4
, newlib
, sources
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

  sdk = stdenv.mkDerivation {
    name = "microkit-sdk";

    # src = microkitSource;

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
      python3 build_sdk.py --sel4=${kernelSourcePatched}
    '';

    installPhase = ''
      mv release/microkit-sdk-1.2.6 $out
    '';
  };

  tool = linkFarm "microkit-tool" [
    (rec {
      name = "microkit";
      path = lib.cleanSource (microkitSource + "/tool/${name}");
      # path = microkitSource + "/tool/${name}";
    })
  ];

  exampleSource = microkitSource + "/example/qemu_virt_aarch64/hello";

  examplePDs = stdenv.mkDerivation {
    name = "example";

    src = exampleSource;

    dontConfigure = true;
    dontFixup = true;

    nativeBuildInputs = [
      python3Packages.sel4-deps
    ];

    MICROKIT_SDK = sdk;
    MICROKIT_BOARD = "qemu_virt_aarch64";
    MICROKIT_CONFIG = "debug";

    MICROKIT_TOOL = "python3 -m microkit";

    buildPhase = ''
      export PYTHONPATH=${tool}:$PYTHONPATH
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
      export PYTHONPATH=${tool}:$PYTHONPATH
      mkdir $out
        python3 -m microkit ${systemXML} \
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
    sdk tool
    mkSystem
    example
  ;
}
