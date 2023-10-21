#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, hostPlatform
, buildPackages
, writeScript
}:

let
  defaultTimeout = 30;

  elaborateSimpleAutomationParams =
    { timeout ? defaultTimeout
    }:
    {
      inherit timeout;
    };

in rec {

  automateQemuSimple = { simulate, timeout }:
    let
      py = buildPackages.python3.withPackages (pkgs: [
        pkgs.pexpect
      ]);
    in
      writeScript "automate-qemu" ''
        #!${buildPackages.runtimeShell}
        set -eu

        ${py}/bin/python3 ${./automate_simple.py} --simulate ${simulate} --timeout ${toString timeout}
      '';

  mkMkInstanceForPlatform =
    { mkQemuCmd
    , platformRequiresLoader ? true
    }:

    { rootTask ? null
    , loader ? null
    , canAutomateSimply ? false
    , simpleAutomationParams ? if canAutomateSimply then {} else null
    , extraQemuArgs ? []
    }:

    let
      qemuCmd = mkQemuCmd (if platformRequiresLoader then loader else rootTask);

      simulate = writeScript "run.sh" ''
        #!${buildPackages.runtimeShell}
        exec ${lib.concatStringsSep " " (qemuCmd ++ extraQemuArgs)} "$@"
      '';

      elaboratedSimpleAutomationParams =
        if simpleAutomationParams == null
        then null
        else elaborateSimpleAutomationParams simpleAutomationParams;

      automate =
        if elaboratedSimpleAutomationParams == null
        then null
        else automateQemuSimple {
          inherit simulate;
          inherit (elaboratedSimpleAutomationParams) timeout;
        };

    in rec {
      attrs = {
        inherit simulate automate;
      };
      links = [
        { name = "simulate"; path = attrs.simulate; }
      ];
    };
}
