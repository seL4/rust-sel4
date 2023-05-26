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
    writeScript "automate-qemu" ''
      #!${buildPackages.runtimeShell}
      set -eu

      script=${simulate}
      timeout_=${toString timeout}

      echo "running '$script' with timeout ''${timeout_}s"

      # the odd structure of this next part is due to bash's limitations on
      # pipes, process substition, and coprocesses.

      coproc $script < /dev/null
      result=$( \
        timeout $timeout_ bash -c \
          'head -n1 <(bash -c "tee >(cat >&2)" | grep -E -a --line-buffered --only-matching "TEST_(PASS|FAIL)")' \
          <&''${COPROC[0]} \
          || true
      )
      kill $COPROC_PID

      echo "result: '$result'"
      [ "$result" == "TEST_PASS" ]
    '';

  mkMkInstanceForPlatform =
    { mkQemuCmd
    , platformRequiresLoader ? true
    }:

    { rootTask ? null
    , loader ? null
    , simpleAutomationParams ? null
    }:

    let
      qemuCmd = mkQemuCmd (if platformRequiresLoader then loader else rootTask);

      simulate = writeScript "run.sh" ''
        #!${buildPackages.runtimeShell}
        exec ${lib.concatStringsSep " " qemuCmd} "$@"
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
