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
, verus
, genSDF

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
    targetTriple = mkSeL4RustTargetTriple { microkit = true; };
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

    banscii = maybe (isMicrokit && seL4Config.PLAT == "qemu-arm-virt") (callPackage ./banscii {
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
    minimal = maybe isMicrokit (
      let
        pd = mkPD rec {
          rootCrate = crates.tests-microkit-minimal;
          targetTriple = mkSeL4RustTargetTriple { microkit = true; minimal = true; };
        };
      in
        callPlatform {
          system = microkit.mkSystem {
            searchPath = [
              "${pd}/bin"
            ];
            systemXML = sources.srcRoot + "/crates/private/tests/microkit/minimal/src/bin/system.xml";
          };
          extraPlatformArgs = lib.optionalAttrs canSimulate  {
            canAutomateSimply = true;
          };
        } // {
          inherit pd;
        }
    );

    unwind = maybe isMicrokit (
      let
        pd = mkPD rec {
          rootCrate = crates.tests-microkit-unwind;
          targetTriple = mkSeL4RustTargetTriple { microkit = true; unwind = true; };
        };
      in
        callPlatform {
          system = microkit.mkSystem {
            searchPath = [
              "${pd}/bin"
            ];
            systemXML = sources.srcRoot + "/crates/private/tests/microkit/unwind/src/bin/system.xml";
          };
          extraPlatformArgs = lib.optionalAttrs canSimulate  {
            canAutomateSimply = true;
          };
        } // {
          inherit pd;
        }
    );

    passive-server-with-deferred-action = maybe isMicrokit (
      let
        mkCrateName = role: "tests-microkit-passive-server-with-deferred-action-pds-${role}";

        pd = mkPD {
          rootCrate = crates.tests-microkit-passive-server-with-deferred-action;
        };
      in
        callPlatform {
          system = microkit.mkSystem {
            searchPath = [
              "${pd}/bin"
            ];
            systemXML = sources.srcRoot + "/crates/private/tests/microkit/passive-server-with-deferred-action/src/bin/test/system.xml";
          };
          extraPlatformArgs = lib.optionalAttrs canSimulate  {
            canAutomateSimply = true;
          };
        } // {
          inherit pd;
        }
    );

    reset = maybe (isMicrokit && stdenv.hostPlatform.isAarch64) (
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
            systemXML = sources.srcRoot + "/crates/private/tests/microkit/reset/src/bin/system.xml";
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
