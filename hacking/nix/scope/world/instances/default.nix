{ lib, hostPlatform, buildPackages
, callPackage
, writeScript, runCommand, linkFarm

, cpio

, sources

, crates
, crateUtils

, mkTask, mkSeL4KernelWithPayload
, embedDebugInfo
, seL4RustTargetInfoWithConfig
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
  inherit (worldConfig) isCorePlatform;

  haveFullRuntime = !isCorePlatform && (hostPlatform.isAarch64 || hostPlatform.isx86_64);
  haveMinimalRuntime = haveFullRuntime;
  haveKernelLoader = hostPlatform.isAarch64;

  maybe = condition: v: if condition then v else null;

  isQEMU = true;

in rec {

  all = lib.filter (v: v != null) [
    tests.root-task.loader
    tests.root-task.core-libs
    tests.root-task.config
    tests.root-task.tls
    tests.root-task.backtrace
    tests.root-task.panicking.abort.withAlloc
    tests.root-task.panicking.abort.withoutAlloc
    tests.root-task.panicking.unwind.withAlloc
    tests.root-task.panicking.unwind.withoutAlloc
    tests.root-task.c
    tests.capdl.http-server
    tests.capdl.threads
    tests.capdl.utcover
    sel4cp.examples.hello.system
    sel4cp.examples.banscii.system
    sel4cp.tests.passive-server-with-deferred-action.system
    examples.root-task.hello
    examples.root-task.example-root-task
    examples.root-task.example-root-task-without-runtime
  ];

  subsets = rec {
    supported = all;
    canAutomate = lib.filter (instance: (instance.automate or null) != null) supported;
  };

  tests = {
    root-task = {
      loader = maybe (haveKernelLoader && haveFullRuntime) (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-loader;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
      });

      core-libs = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-core-libs;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
      });

      config = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-config;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
      });

      tls = maybe haveFullRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.tests-root-task-tls;
          release = false;
        };
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
      });

      backtrace = maybe haveFullRuntime (mkInstance {
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
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
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
        in
          lib.flip lib.mapAttrs panicStrategy
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
                  extraPlatformArgs = lib.optionalAttrs isQEMU  {
                    canAutomateSimply = panicStrategyName == "unwind";
                  };
                })));

      c = maybe haveFullRuntime (callPackage ./c.nix {
        # inherit isQEMU;
      });
    };

    capdl = {
      threads = maybe haveFullRuntime (mkInstance {
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
            };
          };
        };
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
      });

      utcover = maybe haveFullRuntime (mkInstance {
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
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
      });

      http-server =
        maybe
          (hostPlatform.isAarch64 && !isCorePlatform && seL4Config.PLAT == "qemu-arm-virt")
          (callPackage ./http-server {
            inherit isQEMU;
          });
    };
  };

  sel4cp = callPackage ./sel4cp.nix {
    inherit maybe;
    inherit isQEMU;
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
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
      });

      example-root-task-without-runtime = maybe haveMinimalRuntime (mkInstance {
        rootTask = mkTask {
          rootCrate = crates.example-root-task-without-runtime;
          release = false;
          rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = true; };
        };
        extraPlatformArgs = lib.optionalAttrs isQEMU  {
          canAutomateSimply = true;
        };
      });
    };
  };
}
