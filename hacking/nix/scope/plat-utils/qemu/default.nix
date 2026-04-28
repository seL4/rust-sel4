#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, buildPackages
, writeScript
, pathToLink
, selfTopLevel
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

  automateQEMUSimple = { simulate, timeout }:
    let
      py = buildPackages.python3.withPackages (pkgs: [
        pkgs.pexpect
      ]);
      script = buildPackages.writeShellApplication {
        name = "automate-qemu";
        runtimeInputs = [
          selfTopLevel.pkgs.build.coreutils # timeout
          selfTopLevel.pkgs.build.this.sel4-test-sentinels-wrapper
        ];
        # -f to allow ctrl-c
        text = ''
          timeout -f \
            ${toString timeout}s \
            sel4-test-sentinels-wrapper ${simulate}
        '';
      };
    in
      pathToLink "automate-qemu" "${script}/bin/${script.name}";

  mkMkPlatformSystemExtension =
    { mkQEMUCmd
    }:

    { worldConfig
    , rootTaskImage ? null
    , loaderImage ? null
    , extraQEMUArgs ? []
    , canAutomateSimply ? false
    , simpleAutomationParams ? if canAutomateSimply then {} else null
    }:

    let
      qemuCmd = mkQEMUCmd (if worldConfig.platformRequiresLoader then loaderImage else rootTaskImage);

      simulate = writeScript "run.sh" ''
        #!${buildPackages.runtimeShell}
        exec ${lib.concatStringsSep " " (qemuCmd ++ extraQEMUArgs)} "$@"
      '';

      elaboratedSimpleAutomationParams =
        if simpleAutomationParams == null
        then null
        else elaborateSimpleAutomationParams simpleAutomationParams;

      automate =
        if elaboratedSimpleAutomationParams == null
        then null
        else automateQEMUSimple {
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
