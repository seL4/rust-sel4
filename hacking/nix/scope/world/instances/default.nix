#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, hostPlatform, buildPlatform, buildPackages
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
, mkSeL4RustTargetTriple
, worldConfig
, seL4ForBoot
, seL4Config
, seL4RustEnvVars
, seL4Modifications

, muslForSeL4
, dummyLibunwind

, callPlatform
, mkSystem
, mkCapDLInitializer
, mkSimpleCompositionCapDLSpec
}:

let
  inherit (worldConfig) isMicrokit canSimulate;

  haveFullRuntime = !isMicrokit;
  haveMinimalRuntime = !isMicrokit;
  haveUnwindingSupport = !hostPlatform.isAarch32;
  haveKernelLoader = hostPlatform.isAarch || hostPlatform.isRiscV;
  haveCapDLInitializer = true;

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

  mkInstance =
    { rootTask
    , ...
    } @ args:
    callPlatform (
      {
        system = mkSystem {
          inherit rootTask;
        };
      } // builtins.removeAttrs args [ "rootTask" ]
    );

in rec {

  # TODO collect automatically
  all = lib.filter (v: v != null) [
    tests.root-task.loader
    tests.root-task.config
    tests.root-task.tls
    tests.root-task.backtrace
    tests.root-task.panicking
    tests.root-task.c
    tests.root-task.musl
    tests.root-task.verus
    tests.root-task.dafny
    tests.root-task.default-test-harness
    tests.capdl.threads
    tests.capdl.utcover
    microkit.examples.hello
    microkit.examples.banscii
    microkit.examples.http-server
    microkit.tests.minimal
    microkit.tests.passive-server-with-deferred-action
    microkit.tests.reset
    examples.root-task.hello
    examples.root-task.example-root-task
    examples.root-task.example-root-task-without-runtime
    examples.root-task.spawn-thread
    examples.root-task.spawn-task
    examples.root-task.serial-device

    # tests.root-task.ring
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

      config = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-config;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      alloca = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-alloca;
          targetTriple = mkSeL4RustTargetTriple { minimal = true; };
          release = true; # test optimizations
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

      backtrace = maybe (haveFullRuntime && haveUnwindingSupport) (mkInstance rec {
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
        extraDebuggingLinks = [
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

          profile = {
            release = true;
            debug = false;
          };

          byConfig = lib.flip lib.mapAttrs panicStrategy
            (panicStrategyName: _:
              lib.flip lib.mapAttrs alloc
                (_: allocFeatures:
                  lib.flip lib.mapAttrs profile
                    (_: release:
                      let
                        isUnwind = panicStrategyName == "unwind";
                      in
                        maybe (haveFullRuntime && haveUnwindingSupport) (mkInstance {
                          rootTask = mkTask {
                            rootCrate = crates.tests-root-task-panicking;
                            targetTriple = mkSeL4RustTargetTriple { unwind = isUnwind; };
                            inherit release;
                            features = allocFeatures ++ [ "panic-${panicStrategyName}" ];
                          };
                          extraPlatformArgs = lib.optionalAttrs canSimulate {
                            canAutomateSimply = isUnwind;
                          };
                        })
                    )));

          paths = lib.mapCartesianProduct
            ({ panicStrategyName, allocName, profileName }: [ panicStrategyName allocName profileName ])
            (lib.mapAttrs (lib.const lib.attrNames) {
              panicStrategyName = panicStrategy;
              allocName = alloc;
              profileName = profile;
            });

          automate = mkRunTests "run-all-panicking-tests"
            (lib.forEach
              (lib.filter
                (config: config != null)
                (lib.forEach paths (path: lib.attrByPath path (throw "x") byConfig)))
              (config: {
                name = "test"; # TODO
                value = config.automate;
              }));

          simulate = writeText "all-panicking-scripts" (toString (lib.forEach
            (lib.filter
              (config: config != null)
              (lib.forEach paths (path: lib.attrByPath path (throw "x") byConfig)))
            (config: config.simulate)));

          links = linkFarm "links" {
            inherit simulate;
          };

        in {
          inherit byConfig;
          inherit links simulate automate;
        };

      c = maybe haveFullRuntime (callPackage ./c.nix {
        inherit mkInstance canSimulate;
      });

      musl = maybe (haveFullRuntime && !hostPlatform.isRiscV /* TODO riscv linking broken */) (mkInstance {
        rootTask =
          let
            targetTriple = mkSeL4RustTargetTriple { musl = true; };
          in
            mkTask {
              rootCrate = crates.tests-root-task-musl;
              release = false;
              std = true;
              inherit targetTriple;
              commonModifications = {
                modifyConfig = old: lib.recursiveUpdate old {
                  target.${targetTriple.name} = {
                    rustflags = (old.target.${targetTriple.name}.rustflags or []) ++ [
                      "-L" "${muslForSeL4}/lib"
                      "-L" "${dummyLibunwind}/lib"
                    ];
                  };
                };
              };
            };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      verus = maybe (haveFullRuntime && hostPlatform.is64bit && buildPlatform.isx86_64) (callPackage ./verus.nix {
        inherit mkInstance canSimulate;
      });

      dafny = maybe haveFullRuntime (callPackage ./dafny.nix {
        inherit mkInstance canSimulate;
      });

      default-test-harness = maybe (haveFullRuntime && haveUnwindingSupport) (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-default-test-harness;
          targetTriple = mkSeL4RustTargetTriple { unwind = haveUnwindingSupport; };
          test = true;
          justBuildTests = true;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      # ring at 8c665d20ed7621b81d8f4ad564cb7f43a02d42ad (sel4-testing)
      ring = maybe (haveFullRuntime && haveUnwindingSupport && !hostPlatform.isRiscV32 && !hostPlatform.isx86) (
        let
          rootTask = lib.makeOverridable mkTask {
            rootCrate = crates.ring;
            targetTriple = mkSeL4RustTargetTriple { unwind = haveUnwindingSupport; };
            test = true;
            justBuildTests = true;
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

          automate = mkRunTests "run-all-ring-test" (lib.flip lib.mapAttrsToList byElf (k: v: lib.nameValuePair k v.automate));
        }
      );

    };

    capdl = {
      threads = maybe (haveFullRuntime && haveCapDLInitializer) (mkInstance rec {
        test = mkTask {
          rootCrate = crates.tests-capdl-threads-components-test;
          targetTriple = mkSeL4RustTargetTriple { unwind = haveUnwindingSupport; };
          release = true; # test optimizations
        };
        rootTask = mkCapDLInitializer {
          small = false;
          spec = mkSimpleCompositionCapDLSpec {
            script = sources.srcRoot + "/crates/private/tests/capdl/threads/cdl.py";
            config = {
              components = {
                example_component.image = test.elf;
              };
            };
          };
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      utcover = maybe (haveFullRuntime && haveCapDLInitializer) (mkInstance rec {
        test = mkTask {
          rootCrate = crates.tests-capdl-utcover-components-test;
          targetTriple = mkSeL4RustTargetTriple { unwind = haveUnwindingSupport; };
          release = false;
        };
        rootTask = mkCapDLInitializer {
          small = true;
          spec = mkSimpleCompositionCapDLSpec {
            script = sources.srcRoot + "/crates/private/tests/capdl/utcover/cdl.py";
            config = {
              components = {
                example_component.image = test.elf;
              };
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
          targetTriple = mkSeL4RustTargetTriple { minimal = true; };
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      serial-device = maybe (haveFullRuntime && hostPlatform.isAarch) (lib.fix (self: mkInstance {
        rootTask = mkTask {
          rootCrate = crates.serial-device;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
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
              ${py}/bin/python3 ${./test-automation-scripts/serial-device.py} ${self.simulate}
            '';
      }));

      spawn-thread = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.spawn-thread;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs canSimulate {
          canAutomateSimply = true;
        };
      });

      spawn-task = maybe haveFullRuntime (
        let
          child = mkTask {
            rootCrate = crates.spawn-task-child;
            release = false;
          };
        in
          mkInstance {
            rootTask = mkTask {
              rootCrate = crates.spawn-task;
              release = false;
              lastLayerModifications.modifyDerivation = drv: drv.overrideAttrs (attrs: {
                CHILD_ELF = child.split.min;
                passthru = (attrs.passthru or {}) // {
                  inherit child;
                };
              });
            };
            extraPlatformArgs = lib.optionalAttrs canSimulate {
              canAutomateSimply = true;
            };
          }
      );
    };
  };
}
