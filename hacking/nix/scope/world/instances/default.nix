{ lib, hostPlatform, buildPackages
, writeScript, linkFarm
, crates
, mkTask, mkLoader
, embedDebugInfo
, seL4RustTargetInfoWithConfig
, worldConfig
, callPackage
, seL4ForBoot

, mkCapDLLoader
, mkSmallCapDLLoader
, mkSimpleCompositionCapDLSpec

, mkInstance
, mkCapDLRootTask
}:

let
  haveFullRuntime = hostPlatform.isAarch64 || hostPlatform.isx86_64;
  haveMinimalRuntime = haveFullRuntime;
  haveKernelLoader = hostPlatform.isAarch64;

in rec {

  all = [
    examples.root-task.example-root-task
    examples.root-task.example-root-task-without-runtime
    tests.loader
    tests.core-libs
    tests.config
    tests.tls
    tests.injected-phdrs
    tests.backtrace
    tests.panicking.abort.withAlloc
    tests.panicking.abort.withoutAlloc
    tests.panicking.unwind.withAlloc
    tests.panicking.unwind.withoutAlloc
    tests.capdl.threads
    tests.capdl.utcover
  ];

  supported = lib.filter (instance: instance.isSupported) all;

  examples = {
    root-task = {
      example-root-task-without-runtime = mkInstance {
        rootTask = mkTask {
          rootCrate = crates.example-root-task-without-runtime;
          release = false;
          rustTargetInfo = seL4RustTargetInfoWithConfig { minimal = true; };
        };
        isSupported = haveMinimalRuntime;
        canAutomate = true;
      };

      example-root-task = mkInstance {
        rootTask = mkTask {
          rootCrate = crates.example-root-task;
          release = false;
        };
        isSupported = haveFullRuntime;
        canAutomate = true;
      };
    };
  };

  tests = {
    loader = mkInstance {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-loader;
        release = false;
      };
      isSupported = haveKernelLoader && haveFullRuntime;
      canAutomate = true;
    };

    core-libs = mkInstance {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-core-libs;
        release = false;
      };
      isSupported = haveFullRuntime;
      canAutomate = true;
    };

    config = mkInstance {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-config;
        release = false;
      };
      isSupported = haveFullRuntime;
      canAutomate = true;
    };

    tls = mkInstance {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-tls;
        release = false;
      };
      isSupported = haveFullRuntime;
      canAutomate = true;
    };

    injected-phdrs = mkInstance {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-injected-phdrs;
        release = true;
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
      canAutomate = true;
    };

    backtrace = mkInstance {
      rootTask =
        let
          orig = mkTask {
            rootCrate = crates.tests-root-task-backtrace;
            release = false;
          };
        in {
          elf = embedDebugInfo orig.elf;
          inherit orig;
        };
      isSupported = haveFullRuntime;
      canAutomate = true;
    };

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
              (_: allocFeatures: mkInstance {
                rootTask = mkTask {
                  rootCrate = crates.tests-root-task-panicking;
                  release = false;
                  features = allocFeatures ++ [ "panic-${panicStrategyName}" ];
                  extraProfile = {
                    panic = panicStrategyName;
                  };
                };
                isSupported = haveFullRuntime;
                canAutomate = panicStrategyName == "unwind";
              }));

    c = callPackage ./c.nix {
      inherit mkInstance;
    };

    capdl = {
      threads = mkInstance {
        rootTask = mkCapDLRootTask rec {
          # small = true;
          script = ../../../../../crates/tests/capdl/threads/cdl.py;
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
        isSupported = haveFullRuntime;
        canAutomate = true;
      };
      utcover = mkInstance {
        rootTask = mkCapDLRootTask rec {
          # small = true;
          script = ../../../../../crates/tests/capdl/utcover/cdl.py;
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
        isSupported = haveFullRuntime;
        canAutomate = true;
      };
    };
  };

  sel4cp = callPackage ./sel4cp.nix {};

}
