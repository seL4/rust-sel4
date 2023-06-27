{ lib, hostPlatform, buildPackages
, callPackage
, writeScript, runCommand, linkFarm

, cpio

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

in rec {

  all = [
    tests.root-task.loader
    tests.root-task.core-libs
    tests.root-task.config
    tests.root-task.tls
    tests.root-task.backtrace
    tests.root-task.panicking.abort.withAlloc
    tests.root-task.panicking.abort.withoutAlloc
    tests.root-task.panicking.unwind.withAlloc
    tests.root-task.panicking.unwind.withoutAlloc
    tests.root-task.c.instance
    tests.capdl.http-server
    tests.capdl.threads
    tests.capdl.utcover
    sel4cp.hello.system
    sel4cp.banscii.system
    examples.root-task.example-root-task
    examples.root-task.example-root-task-without-runtime
  ];

  subsets = rec {
    supported = lib.filter (instance: instance.isSupported) all;
    canAutomate = lib.filter (instance: instance.canAutomate) supported;
  };

  tests = {
    root-task = {
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

      backtrace = mkInstance {
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
    };

    capdl = {
      threads = mkInstance {
        rootTask = mkCapDLRootTask rec {
          small = true;
          script = ../../../../../crates/private/tests/capdl/threads/cdl.py;
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
          script = ../../../../../crates/private/tests/capdl/utcover/cdl.py;
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

      http-server =
        let
          content = builtins.fetchGit {
            url = "https://github.com/seL4/website_pr_hosting";
            ref = "PR_280";
            rev = "0a579415c4837c96c4d4629e4b4d4691aaff07ca";
          };

          contentCPIO = runCommand "x.cpio" {
            nativeBuildInputs = [ cpio ];
          } ''
            cd ${content}/localhost \
              && find . -print -depth \
              | cpio -o -H newc > $out
          '';

        in
          mkInstance {
            rootTask = mkCapDLRootTask rec {
              # small = true;
              script = ../../../../../crates/private/tests/capdl/http-server/cdl.py;
              config = {
                components = {
                  example_component.image = passthru.test.elf;
                  example_component.heap_size = 1073741824 / 4;
                };
              };
              passthru = {
                inherit contentCPIO;
                test = mkTask {
                  rootCrate = crates.tests-capdl-http-server-components-test;
                  # release = false;
                  layers = [
                    crateUtils.defaultIntermediateLayer
                    {
                      crates = [
                        "sel4-simple-task-runtime"
                      ];
                      modifications = {
                        modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars);
                      };
                    }
                  ];
                  # lastLayerModifications = {
                  #   modifyDerivation = drv: drv.overrideAttrs (self: super: {
                  #     CONTENT_CPIO = contentCPIO;
                  #   });
                  # };
                };
              };
            };
            extraPlatformArgs.extraQemuArgs = [
              "-device" "virtio-net-device,netdev=netdev0"
              "-netdev" "user,id=netdev0,hostfwd=tcp::8000-:80"

              "-device" "virtio-blk-device,drive=blkdev0"
              "-blockdev" "node-name=blkdev0,read-only=on,driver=file,filename=${contentCPIO}"
            ];
            isSupported = hostPlatform.isAarch64 && !isCorePlatform && seL4Config.PLAT == "qemu-arm-virt";
            # canAutomate = true;
            # isSupported = false;
            canAutomate = false;
          };
    };
  };

  sel4cp = callPackage ./sel4cp.nix {};

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
}
