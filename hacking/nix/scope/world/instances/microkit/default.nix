#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, symlinkJoin, writeText, runCommand
, callPackage
, microkit
, mkTask
, sources
, crates
, crateUtils
, mkSeL4RustTargetTriple
, mkMicrokitInstance
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
    targetTriple = mkSeL4RustTargetTriple { microkit = true; minimal = true; };
  } // args);

  inherit (worldConfig) isMicrokit;

in {
  examples = {
    hello = maybe isMicrokit (
      let
        pds = {
          hello = mkPD {
            rootCrate = crates.microkit-hello;
            release = false;
          };
        };
      in
        mkMicrokitInstance {
          system = microkit.mkSystem {
            searchPath = "${pds.hello}/bin";
            systemXML = sources.srcRoot + "/crates/examples/microkit/hello/hello.system";
          };
        } // {
          inherit pds;
        }
    );

    banscii = maybe isMicrokit (callPackage ./banscii {
      inherit canSimulate;
      inherit mkPD;
    });

    http-server =
      maybe
        (isMicrokit && seL4Config.PLAT == "qemu-arm-virt")
        (callPackage ./http-server {
          inherit canSimulate;
          inherit mkPD;
        });
  };

  tests = {
    passive-server-with-deferred-action = maybe isMicrokit (
      let
        mkCrateName = role: "tests-microkit-passive-server-with-deferred-action-pds-${role}";

        pds = {
          client = mkPD rec {
            rootCrate = crates.${mkCrateName "client"};
          };
          server = mkPD rec {
            rootCrate = crates.${mkCrateName "server"};
          };
        };
      in
        mkMicrokitInstance {
          system = microkit.mkSystem {
            searchPath = symlinkJoin {
              name = "x";
              paths = [
                "${pds.client}/bin"
                "${pds.server}/bin"
              ];
            };
            systemXML = sources.srcRoot + "/crates/private/tests/microkit/passive-server-with-deferred-action/x.system";
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
