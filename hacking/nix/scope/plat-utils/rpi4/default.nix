#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, hostPlatform, callPackage
, pkgsBuildBuild
, runCommand, writeText
, fetchFromGitHub
, ubootTools

, fetchzip
, stdenvNoCC, mtools, utillinux

, platUtils
}:

let
  firmware = fetchFromGitHub {
    owner = "raspberrypi";
    repo = "firmware";
    rev = "1.20230405";
    sha256 = "sha256-UtUd1MbsrDFxd/1C3eOAMDKPZMx+kSMFYOJP+Kc6IU8=";
  };

  baseBootDir = "${firmware}/boot";

  configTxt = writeText "config.txt" ''
    enable_uart=1
    arm_64bit=${if hostPlatform.is32bit then "0" else "1"}
  '';
    # for debugging:
    # start_debug=1
    # uart_2ndstage=1

  uBoot = callPackage ./u-boot.nix {};

  uBootBin = "${uBoot}/u-boot.bin";

  imageAddr = "0x30000000";

  uBootEnvTxt = writeText "uboot.env.txt" ''
    bootdelay=0
    bootcmd=load mmc 0:1 ${imageAddr} /image.elf; bootelf -p ${imageAddr}"
    autostart=1
  '';

  uBootEnv = runCommand "uboot.env" {
    nativeBuildInputs = [
      ubootTools
    ];
  } ''
    mkenvimage \
      -s 0x4000 \
      -o $out \
      ${uBootEnvTxt}
  '';

  kernelFileName = "kernel${if hostPlatform.is32bit then "7l" else "8"}.img";

  mkBootLinks =
    { image ? null
    , extraCommands ? ""
    }:
    runCommand "boot" {} ''
      mkdir $out
      ln -s ${baseBootDir}/*.* $out
      mkdir $out/overlays
      ln -s ${baseBootDir}/overlays/*.* $out/overlays

      ln -sf ${configTxt} $out/config.txt

      rm $out/kernel*.img
      ln -s ${uBootBin} $out/${kernelFileName}

      ln -s ${uBootEnv} $out/uboot.env

      ${lib.optionalString (image != null) ''
        ln -s ${image} $out/image.elf
      ''}

      ${extraCommands}
    '';

  mkBootCopied = bootLinks: runCommand "boot" {} ''
    cp -rL ${bootLinks} $out
  '';

  defaultBootLinks = mkBootLinks {};

  mkInstanceForPlatform =
    { loader
    , rootTask
    , bootLinksExtraCommands ? ""
    , simpleAutomationParams ? null # TODO
    }:
    let
      boot = mkBootLinks {
        image = loader;
        extraCommands = bootLinksExtraCommands;
      };
      bootCopied = mkBootCopied boot;
      qemu = platUtils.qemu.mkMkInstanceForPlatform {
        mkQemuCmd = loader: [
          "${pkgsBuildBuild.this.qemuForSeL4}/bin/qemu-system-${if hostPlatform.is32bit then "arm" else "aarch64"}"
            "-smp" "4"
            "-m" "size=2048"
            "-machine" "raspi4b"
            "-nographic"
            "-serial" "null"
            "-serial" "mon:stdio"
            "-kernel" loader
        ];
      } {
        inherit loader rootTask;
      };
    in platUtils.composeInstanceForPlatformAttrs qemu (rec {
      attrs = {
        inherit boot bootCopied;
      };
      links = [
        { name = "boot";
          path = boot;
        }
      ];
    });

in {
  inherit
    firmware
    uBoot
    defaultBootLinks
    mkInstanceForPlatform
  ;
}
