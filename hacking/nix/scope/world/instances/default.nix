#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, hostPlatform, buildPackages
, callPackage
, runCommand, linkFarm, writeText, writeScript

, cpio
, cmake, perl, python3Packages
, breakpointHook, bashInteractive

, sources

, crates
, crateUtils

, mkTask, mkSeL4KernelLoaderWithPayload
, embedDebugInfo
, seL4RustTargetInfoWithConfig, defaultRustTargetInfo
, worldConfig
, seL4ForBoot
, seL4Config
, seL4RustEnvVars

, mkCapDLInitializer
, mkSmallCapDLInitializer
, mkSimpleCompositionCapDLSpec

, mkInstance
, mkCapDLRootTask
}:

let
  inherit (worldConfig) isMicrokit canSimulate;

  haveFullRuntime = !isMicrokit && (hostPlatform.isAarch64 || hostPlatform.isRiscV || hostPlatform.isx86_64);
  haveMinimalRuntime = haveFullRuntime;
  haveKernelLoader = hostPlatform.isAarch64 || hostPlatform.isRiscV;
  haveCapDLInitializer = hostPlatform.isAarch64 || hostPlatform.isRiscV64 || hostPlatform.isx86_64;

  maybe = condition: v: if condition then v else null;

  mkRunTests = name: tests: writeScript name ''
    #!${buildPackages.runtimeShell}
    set -eu

    say() {
      printf '\n<<< %s >>>\n\n' "$1"
    }

    ${lib.concatStrings (lib.forEach tests ({ name, value }: ''
      say 'running test: ${name}'
      ${
        if value != null
        then value
        else ''
          say '(skipping)'
        ''
      }
    ''))}

    say 'all tests passed'
  '';

in rec {

  # TODO collect automatically
  all = lib.filter (v: v != null) [
    tests.root-task.loader
    tests.root-task.core-libs
    tests.root-task.config
    tests.root-task.tls
    tests.root-task.backtrace
    tests.root-task.panicking.byConfig.abort.withAlloc
    tests.root-task.panicking.byConfig.abort.withoutAlloc
    tests.root-task.panicking.byConfig.unwind.withAlloc
    tests.root-task.panicking.byConfig.unwind.withoutAlloc
    tests.root-task.default-test-harness
    tests.root-task.c
    tests.capdl.threads
    tests.capdl.utcover
    microkit.examples.hello
    microkit.examples.banscii
    microkit.examples.http-server
    microkit.tests.passive-server-with-deferred-action
    examples.root-task.hello
    examples.root-task.example-root-task
    examples.root-task.example-root-task-without-runtime
  ];

  allAutomationScripts = map
    (instance: instance.automate)
    (lib.filter (instance: (instance.automate or null) != null) all);

  tests = {
    root-task = rec {
      loader = maybe (haveKernelLoader && haveFullRuntime) (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-loader;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      core-libs = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-core-libs;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      config = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-config;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      tls = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-tls;
          release = true; # test optimizations
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      backtrace = maybe haveFullRuntime (mkInstance rec {
        rootTask =
          let
            orig = mkTask {
              rootCrate = crates.tests-root-task-backtrace;
              release = false;
              extraProfile = {
                opt-level = 2;
              };
            };
          in {
            elf = embedDebugInfo orig.elf;
            inherit orig;
          };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
        extraLinks = [
          { name = "root-task.orig.elf"; path = rootTask.orig.elf; }
        ];
      });

      panicking =
        let
          alloc = {
            withAlloc = [ "alloc" ];
            withoutAlloc = [];
          };

          panicStrategy = {
            unwind = null;
            abort = null;
          };

          byConfig = lib.flip lib.mapAttrs panicStrategy
            (panicStrategyName: _:
              lib.flip lib.mapAttrs alloc
                (_: allocFeatures: maybe haveFullRuntime (mkInstance {
                  rootTask = mkTask {
                    rootCrate = crates.tests-root-task-panicking;
                    release = false;
                    features = allocFeatures ++ [ "panic-${panicStrategyName}" ];
                    extraProfile = {
                      panic = panicStrategyName;
                    };
                  };
                  extraPlatformArgs = lib.optionalAttrs canSimulate {
                    canAutomateSimply = panicStrategyName == "unwind";
                  };
                })));

          paths = [
            [ "abort" "withAlloc" ]
            [ "abort" "withoutAlloc" ]
            [ "unwind" "withAlloc" ]
            [ "unwind" "withoutAlloc" ]
          ];

          automate = mkRunTests "run-all-panicking-tests" (lib.forEach paths (path: {
            name = lib.concatStringsSep "." path;
            value = (lib.attrByPath path (throw "x") byConfig).automate;
          }));

          simulate = writeText "all-panicking-scripts" (toString (lib.forEach paths (path:
            (lib.attrByPath path (throw "x") byConfig).simulate
          )));

        in {
          inherit byConfig automate simulate;
        };

      c = maybe (haveFullRuntime && hostPlatform.isAarch64) (callPackage ./c.nix {
        inherit canSimulate;
      });

      default-test-harness = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-default-test-harness;
          test = true;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      ring = maybe haveFullRuntime (
        let
          rootTask = lib.makeOverridable mkTask {
            rootCrate = crates.ring;
            test = true;
            features = [
              "less-safe-getrandom-custom-or-rdrand"
              # "slow_tests"
            ];
            release = true;
            lastLayerModifications.modifyDerivation = drv: drv.overrideAttrs (attrs: {
              nativeBuildInputs = (attrs.nativeBuildInputs or []) ++ [
                perl
              ];
            });
          };

          fnamesFile = runCommand "elfs.txt" {} ''
            cd ${rootTask}/bin
            echo -n *.elf > $out
          '';

          fnames = lib.splitString " " (builtins.readFile fnamesFile);

          byElf = lib.listToAttrs (lib.forEach fnames (fname:
            let
              name = lib.head (lib.splitString "." fname);
            in
              lib.nameValuePair name (mkInstance {
                rootTask = rootTask.override {
                  getELF = drv: runCommand "test.elf" {} ''
                    ln -s ${drv}/bin/${fname} $out
                  '';
                };
                extraPlatformArgs = lib.optionalAttrs canSimulate {
                  canAutomateSimply = true;
                  simpleAutomationParams.timeout = 10 * 60;
                };
              }
            )
          ));
        in {
          inherit byElf;

          all = mkRunTests "run-all-ring-test" (lib.flip lib.mapAttrsToList byElf (k: v: lib.nameValuePair k v.automate));
        }
      );

    };

    capdl = {
      threads = maybe (haveFullRuntime && haveCapDLInitializer) (mkInstance {
        rootTask = mkCapDLRootTask rec {
          small = true;
          script = sources.srcRoot + "/crates/private/tests/capdl/threads/cdl.py";
          config = {
            components = {
              example_component.image = passthru.test.elf;
            };
          };
          passthru = {
            test = mkTask {
              rootCrate = crates.tests-capdl-threads-components-test;
              release = true; # test optimizations
            };
          };
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      utcover = maybe (haveFullRuntime && haveCapDLInitializer) (mkInstance {
        rootTask = mkCapDLRootTask rec {
          # small = true;
          script = sources.srcRoot + "/crates/private/tests/capdl/utcover/cdl.py";
          config = {
            components = {
              example_component.image = passthru.test.elf;
            };
          };
          passthru = {
            test = mkTask {
              rootCrate = crates.tests-capdl-utcover-components-test;
              release = false;
            };
          };
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });
    };
  };

  microkit = callPackage ./microkit {
    inherit maybe;
    inherit canSimulate;
  };

  examples = {
    root-task = {
      hello = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.hello;
          release = false;
        };
      });
    };

    root-task = {
      example-root-task = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.example-root-task;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      example-root-task-without-runtime = maybe haveMinimalRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.example-root-task-without-runtime;
          release = false;
          rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = true; };
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });
    };
  };
}
