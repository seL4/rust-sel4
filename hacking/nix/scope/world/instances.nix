{ lib, hostPlatform, buildPackages
, writeScript, linkFarm
, crates
, mkTask, mkLoader
, worldConfig
}:

let
  mk = { rootTask, isSupported }: rec {
    inherit rootTask isSupported;

    loader = mkLoader {
      appELF = rootTask.elf;
    };

    qemuCmd = worldConfig.mkQemuCmd (if worldConfig.qemuCmdRequiresLoader then loader.elf else rootTask.elf);

    simulate = writeScript "run.sh" ''
      #!${buildPackages.runtimeShell}
      exec ${lib.concatStringsSep " " qemuCmd}
    '';

    symbolizeRootTaskBacktrace = writeScript "x.sh" ''
        #!${buildPackages.runtimeShell}
        exec ${buildPackages.this.sel4-symbolize-backtrace}/bin/sel4-symbolize-backtrace -f ${rootTask.elf} "$@"
    '';

    links = linkFarm "links" ([
      { name = "simulate"; path = simulate; }
    ] ++ lib.optionals worldConfig.qemuCmdRequiresLoader [
      { name = "loader.elf"; path = loader.elf; }
    ] ++ [
      { name = "root-task.elf"; path = rootTask.elf; }
      { name = "symbolize-root-task-backtrace"; path = symbolizeRootTaskBacktrace; }
    ]);
  };

  haveFullRuntime = hostPlatform.isAarch64;
  haveMinimalRuntime = haveFullRuntime || hostPlatform.isx86_64;

in rec {

  all = [
    examples.minimal-runtime-with-state
    examples.minimal-runtime-without-state
    examples.full-runtime
    tests.loader
    tests.core-libs
    tests.config
    tests.tls
    tests.backtrace
    tests.panicking.abort.withAlloc
    tests.panicking.abort.withoutAlloc
    tests.panicking.unwind.withAlloc
    tests.panicking.unwind.withoutAlloc
  ];

  supported = lib.filter (instance: instance.isSupported) all;

  examples = {
    minimal-runtime-with-state = mk {
      rootTask = mkTask {
        rootCrate = crates.minimal-runtime-with-state;
        release = false;
        extraProfile = {
          panic = "abort";
        };
      };
      isSupported = haveMinimalRuntime;
    };

    minimal-runtime-without-state = mk {
      rootTask = mkTask {
        rootCrate = crates.minimal-runtime-without-state;
        release = false;
        extraProfile = {
          panic = "abort";
        };
      };
      isSupported = haveMinimalRuntime;
    };

    full-runtime = mk {
      rootTask = mkTask {
        rootCrate = crates.full-runtime;
        injectPhdrs = true;
        release = false;
      };
      isSupported = haveFullRuntime;
    };
  };

  tests = {
    loader = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-loader;
        release = false;
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
    };

    core-libs = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-core-libs;
        release = false;
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
    };

    config = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-config;
        release = false;
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
    };

    tls = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-tls;
        release = false;
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
    };

    backtrace = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-backtrace;
        release = false;
        rootTaskStackSize = 4096 * 64; # TODO
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
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
              (_: allocFeatures: mk {
                rootTask = mkTask {
                  rootCrate = crates.tests-root-task-panicking;
                  features = allocFeatures ++ [ "panic-${panicStrategyName}" ];
                  extraProfile = {
                    panic = panicStrategyName;
                  };
                  rootTaskStackSize = 4096 * 64; # TODO
                  injectPhdrs = true;
                };
                isSupported = haveFullRuntime;
              }));

  };

}
