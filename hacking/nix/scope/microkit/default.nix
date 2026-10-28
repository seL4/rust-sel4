#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPackages
, linkFarm, writeScript, runCommand
, buildEnv
, callPackage
, cmake, ninja
, dtc, libxml2
, python312Packages
, fenix
, vendorLockfile, vendorSysrootLockfile, mergeVendoredLockfiles
, libclangPath
, qemuForSeL4
, sources
, toTOMLFile
, selfTopLevel
}:

let
  microkitSource = sources.microkit;

  kernelSource = sources.seL4;

  genSDF = callPackage ./gen-sdf.nix {};

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

  rustToolchain = fenix.fromToolchainFile {
    file = ./rust-toolchain.toml;
    sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
  };

  mkSdkCommon =
    { extraDepsBuildBuild ? []
    , extraArgs ? []
    }:

    let
      vendored = mergeVendoredLockfiles [
        (vendorSysrootLockfile rustToolchain)
        (vendorLockfile {
          inherit rustToolchain;
          lockfile = microkitSource + "/Cargo.lock";
        })
      ];

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
        ] ++ extraDepsBuildBuild;

        dontFixup = true;

        patchPhase = ''
          # TODO HACK

          substituteInPlace build_sdk.py \
            --replace \
              'frozenset(args.boards.split(","))' \
              'frozenset(filter(lambda x: x != "", args.boards.split(",")))'

          substituteInPlace build_sdk.py \
            --replace \
              'frozenset(args.configs.split(","))' \
              'frozenset(filter(lambda x: x != "", args.configs.split(",")))'
        '';

        configurePhase = ''
          cat ${toTOMLFile "x" vendored.configFragment} >> .cargo/config.toml
        '';

        buildPhase = ''
          python3 build_sdk.py \
            --sel4 ${kernelSourcePatched} \
            --skip-docs \
            --skip-tar \
            ${lib.concatStringsSep " " extraArgs}
        '';

        installPhase = ''
          mv release/microkit-sdk-* $out
        '';
      };

  sdkArch = stdenv':
    if stdenv'.hostPlatform.isAarch64 then "aarch64"
    else if stdenv'.hostPlatform.isRiscV64 then "riscv64"
    else if stdenv'.hostPlatform.isx86_64 then "x86_64"
    else throw "unknown arch";

  toolchainPrefixArgs = stdenv': [
    "--gcc-toolchain-prefix-${sdkArch stdenv'}" (lib.removeSuffix "-" stdenv'.cc.targetPrefix)
  ];

  sdk =
    let
      stdenvs = map (pkgs: pkgs.stdenv) [
        selfTopLevel.pkgs.host.aarch64.none
        selfTopLevel.pkgs.host.riscv64.default.none
        selfTopLevel.pkgs.host.x86_64.none
      ];
    in
      mkSdkCommon {
        extraDepsBuildBuild = lib.flip lib.concatMap stdenvs (stdenv': [
          stdenv'.cc.bintools.bintools
          stdenv'.cc.cc
        ]);
        extraArgs = lib.concatMap toolchainPrefixArgs stdenvs;
      };

  tool = mkSdkCommon {
    extraArgs = [
      "--boards" "''"
      "--configs" "''"
    ];
  };

  forWorld = worldConfig:
    let
      inherit (worldConfig) platformRequiresLoader microkitConfig;
      inherit (microkitConfig) board config;

      sdkArch =
        if stdenv.hostPlatform.isAarch64 then "aarch64"
        else if stdenv.hostPlatform.isRiscV64 then "riscv64"
        else if stdenv.hostPlatform.isx86_64 then "x86_64"
        else throw "unknown arch";

      sdk = mkSdkCommon {
        extraArgs = [
          "--skip-tool"
          "--boards" board
          "--configs" config
        ] ++ toolchainPrefixArgs stdenv;
      };

      imageName = if platformRequiresLoader then "loader" else "root-task";

      mkSystemImage =
        { systemXML
        , searchPath
        }:
        lib.fix (self: runCommand "system" {
          nativeBuildInputs = [
            tool
          ];
          passthru = {
            inherit systemXML;
            image = "${self}/${imageName}.img";
          };
        } ''
          mkdir $out
          MICROKIT_SDK=${sdk} \
            microkit ${systemXML} \
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

    in {
      inherit
        sdk
        mkSystem
      ;
    };

in {
  inherit
    sdk
    tool
    genSDF
    forWorld
  ;
}
