{ lib, callPackage
, runCommand, writeText
, fetchFromGitHub
, ubootTools

, fetchzip
, stdenvNoCC, mtools, utillinux
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
    arm_64bit=1
  '';
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
      ln -s ${uBootBin} $out/kernel8.img

      ln -s ${uBootEnv} $out/uboot.env

      ${lib.optionalString (image != null) ''
        ln -s ${image} $out/image.elf
      ''}

      ${extraCommands}
    '';

  defaultBootLinks = mkBootLinks {};

  mkInstanceForPlatform =
    { loader
    , rootTask
    , simpleAutomationParams ? null # TODO
    }:
    let
      boot = mkBootLinks {
        image = loader;
      };
    in rec {
      attrs = {
        inherit boot;
      };
      links = [
        { name = "boot";
          path = boot;
        }
      ];
    };

in {
  inherit
    firmware
    uBoot
    defaultBootLinks
    mkInstanceForPlatform
  ;
}
