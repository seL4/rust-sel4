{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, symlinkJoin, writeText, runCommand
, callPackage
, sel4cp
, mkTask
, sources
, crates
, crateUtils
, seL4RustTargetInfoWithConfig
, mkCorePlatformInstance
, worldConfig
, seL4Config

, maybe
, canSimulate
}:

let
  mkPD = args: mkTask (rec {
    # layers = [
    #   crateUtils.defaultIntermediateLayer
    #   {
    #     crates = [ "sel4" ];
    #     modifications = {
    #       modifyDerivation = drv: drv.overrideAttrs (self: super: {
    #         SEL4_PREFIX = seL4ForUserspace;
    #       });
    #     };
    #   }
    # ];
    rustTargetInfo = seL4RustTargetInfoWithConfig { cp = true; minimal = true; };
  } // args);

  inherit (worldConfig) isCorePlatform;

in {
  examples = {
    hello = maybe isCorePlatform (
      let
        pds = {
          hello = mkPD {
            rootCrate = crates.sel4cp-hello;
            release = false;
          };
        };
      in
        mkCorePlatformInstance {
          system = sel4cp.mkSystem {
            searchPath = "${pds.hello}/bin";
            systemXML = sources.srcRoot + "/crates/examples/sel4cp/hello/hello.system";
          };
        } // {
          inherit pds;
        }
    );

    banscii = maybe isCorePlatform (callPackage ./banscii {
      inherit canSimulate;
      inherit mkPD;
    });

    http-server =
      maybe
        (isCorePlatform && seL4Config.PLAT == "qemu-arm-virt")
        (callPackage ./http-server {
          inherit canSimulate;
          inherit mkPD;
        });
  };

  tests = {
    passive-server-with-deferred-action = maybe isCorePlatform (
      let
        mkCrateName = role: "tests-sel4cp-passive-server-with-deferred-action-pds-${role}";

        pds = {
          client = mkPD rec {
            rootCrate = crates.${mkCrateName "client"};
          };
          server = mkPD rec {
            rootCrate = crates.${mkCrateName "server"};
          };
        };
      in
        mkCorePlatformInstance {
          system = sel4cp.mkSystem {
            searchPath = symlinkJoin {
              name = "x";
              paths = [
                "${pds.client}/bin"
                "${pds.server}/bin"
              ];
            };
            systemXML = sources.srcRoot + "/crates/private/tests/sel4cp/passive-server-with-deferred-action/x.system";
          };
          extraPlatformArgs = lib.optionalAttrs canSimulate  {
            canAutomateSimply = true;
          };
        } // {
          inherit pds;
        }
    );
  };
}
