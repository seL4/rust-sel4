{ lib
, hostPlatform
, buildPackages
, writeScript
}:

{

  mkMkInstanceForPlatform =
    { mkQemuCmd
    , platformRequiresLoader ? true
    }:

    { rootTask ? null
    , loader ? null
    }:

    let
      qemuCmd = mkQemuCmd (if platformRequiresLoader then loader else rootTask);
    in rec {
      attrs = {
        simulate = writeScript "run.sh" ''
          #!${buildPackages.runtimeShell}
          exec ${lib.concatStringsSep " " qemuCmd} "$@"
        '';
      };
      links = [
        { name = "simulate"; path = attrs.simulate; }
      ];
    };

}
