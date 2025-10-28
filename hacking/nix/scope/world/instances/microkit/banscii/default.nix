#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, symlinkJoin, writeText, writeScript, runCommand
, python3Packages
, microkit
, sdfgen
, mkTask
, sources
, crates
, crateUtils
, mkSeL4RustTargetTriple
, worldConfig
, seL4ConfigJSON
, callPlatform

, canSimulate
, mkPD
}:

let
  inherit (worldConfig.microkitConfig) board;

  pds = {
    serial-driver = mkPD rec {
      rootCrate = crates.banscii-serial-driver;
      features = [ "board-${board}" ];
    };
    assistant = mkPD rec {
      rootCrate = crates.banscii-assistant;
      release = true;
      targetTriple = mkSeL4RustTargetTriple { microkit = true; minimal = false; };
    };
    artist = mkPD rec {
      rootCrate = crates.banscii-artist;
      release = true;
      extraProfile = {
        # For RSA key generation
        build-override = {
          opt-level = 2;
        };
      };
    };
  };

  srcPath = relativePath: sources.srcRoot + "/crates/examples/microkit/banscii/${relativePath}";

in
lib.fix (self: callPlatform {
  system = microkit.mkSystem {
    searchPath = [
      "${pds.serial-driver}/bin"
      "${pds.assistant}/bin"
      "${pds.artist}/bin"
    ];
    systemXML = runCommand "banscii.system" {
      nativeBuildInputs = [
        sdfgen
      ];
    } ''
      python3 ${srcPath "meta.py"} \
        --board ${board} \
        -o $out
    '';
  };
} // {
  inherit pds;
} // lib.optionalAttrs canSimulate rec {
  automate =
    let
      py = buildPackages.python3.withPackages (pkgs: [
        pkgs.pexpect
      ]);
    in
      writeScript "automate" ''
        #!${buildPackages.runtimeShell}
        set -eu
        ${py}/bin/python3 ${./automate.py} ${self.simulate}
      '';
})
