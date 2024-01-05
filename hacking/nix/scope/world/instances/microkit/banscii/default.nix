#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, symlinkJoin, writeText, writeScript, runCommand
, callPackage
, microkit
, mkTask
, sources
, crates
, crateUtils
, seL4RustTargetInfoWithConfig
, mkMicrokitInstance
, worldConfig

, canSimulate
, mkPD
}:

let
  pds = {
    pl011-driver = mkPD rec {
      rootCrate = crates.banscii-pl011-driver;
      release = true;
    };
    assistant = mkPD rec {
      rootCrate = crates.banscii-assistant;
      release = true;
      rustTargetInfo = seL4RustTargetInfoWithConfig { microkit = true; minimal = false; };
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

in
lib.fix (self: mkMicrokitInstance {
  system = microkit.mkSystem {
    searchPath = symlinkJoin {
      name = "x";
      paths = [
        "${pds.pl011-driver}/bin"
        "${pds.assistant}/bin"
        "${pds.artist}/bin"
      ];
    };
    systemXML = sources.srcRoot + "/crates/examples/microkit/banscii/banscii.system";
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
