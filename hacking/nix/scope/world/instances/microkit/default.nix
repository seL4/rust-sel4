#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, symlinkJoin, writeText, runCommand, runCommandCC
, callPackage
, microkit
, mkTask
, prepareResettable
, sources
, crates
, crateUtils
, mkSeL4RustTargetTriple
, worldConfig
, seL4Config
, callPlatform

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
        callPlatform {
          system = microkit.mkSystem {
            searchPath =  [ "${pds.hello}/bin" ];
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
        callPlatform {
          system = microkit.mkSystem {
            searchPath = [
              "${pds.client}/bin"
              "${pds.server}/bin"
            ];
            systemXML = sources.srcRoot + "/crates/private/tests/microkit/passive-server-with-deferred-action/x.system";
          };
          extraPlatformArgs = lib.optionalAttrs canSimulate  {
            canAutomateSimply = true;
          };
        } // {
          inherit pds;
        }
    );

    reset = maybe isMicrokit (
      let
        pd = rec {
          orig = mkPD rec {
            rootCrate = crates.tests-microkit-reset;
            targetTriple = mkSeL4RustTargetTriple {
              microkit = true;
              resettable = true;
              minimal = false;
            };
            release = false;
          };

          origELF = "${orig}/bin/test.elf";

          patched = prepareResettable origELF;

          sup = runCommandCC "test.sup.elf" {} ''
            $OBJCOPY --only-keep-debug ${origELF} $out
          '';
        };
      in
        callPlatform {
          system = microkit.mkSystem {
            searchPath = [
              (linkFarm "pd" {
                "test.elf" = pd.patched;
              })
              (linkFarm "pd" {
                "test.sup.elf" = pd.sup;
              })
            ];
            systemXML = sources.srcRoot + "/crates/private/tests/microkit/reset/x.system";
          };
          extraPlatformArgs = lib.optionalAttrs canSimulate  {
            canAutomateSimply = true;
          };
          extraDebuggingLinks = [
            { name = "test.orig.elf"; path = pd.origELF; }
            { name = "test.patched.elf"; path = pd.patched; }
            { name = "test.sup.elf"; path = pd.sup; }
          ];
        } // {
          inherit pd;
        }
    );
  };
}
