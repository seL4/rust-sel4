{ lib, hostPlatform, buildPackages
, writeScript, linkFarm
, crates
, mkTask, mkLoader
, worldConfig
}:

let
  mk = { rootTask, isSupported, canAutomate ? false, automateTimeout ? defaultTimeout }: rec {
    inherit rootTask isSupported canAutomate;

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

    automate =
      assert canAutomate;
      assert automateTimeout != null;
      automateQemuBasic {
        inherit simulate;
        timeout = automateTimeout;
      };
  };

  haveFullRuntime = hostPlatform.isAarch64;
  haveMinimalRuntime = haveFullRuntime || hostPlatform.isx86_64;

  automateQemuBasic = { simulate, timeout }:
    writeScript "automate-qemu" ''
      #!${buildPackages.runtimeShell}
      set -eu

      script=${simulate}
      timeout_=${toString timeout}

      echo "running '$script' with timeout ''${timeout_}s"

      # the odd structure of this next part is due to bash's limitations on
      # pipes, process substition, and coprocesses.

      coproc $script < /dev/null
      result=$( \
        timeout $timeout_ bash -c \
          'head -n1 <(bash -c "tee >(cat >&2)" | grep -E -a --line-buffered --only-matching "TEST_(PASS|FAIL)")' \
          <&''${COPROC[0]} \
          || true
      )
      kill $COPROC_PID

      echo "result: '$result'"
      [ "$result" == "TEST_PASS" ]
    '';

  defaultTimeout = 30;

in rec {

  all = [
    examples.root-task.with-full-runtime
    examples.root-task.with-minimal-runtime-with-state
    examples.root-task.with-minimal-runtime-without-state
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
    root-task = {
      with-minimal-runtime-with-state = mk {
        rootTask = mkTask {
          rootCrate = crates.root-task-with-minimal-runtime-with-state;
          release = false;
          extraProfile = {
            panic = "abort";
          };
        };
        isSupported = haveMinimalRuntime;
        canAutomate = true;
      };

      with-minimal-runtime-without-state = mk {
        rootTask = mkTask {
          rootCrate = crates.root-task-with-minimal-runtime-without-state;
          release = false;
          extraProfile = {
            panic = "abort";
          };
        };
        isSupported = haveMinimalRuntime;
        canAutomate = true;
      };

      with-full-runtime = mk {
        rootTask = mkTask {
          rootCrate = crates.root-task-with-full-runtime;
          injectPhdrs = true;
          release = false;
        };
        isSupported = haveFullRuntime;
        canAutomate = true;
      };
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
      canAutomate = true;
    };

    core-libs = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-core-libs;
        release = false;
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
      canAutomate = true;
    };

    config = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-config;
        release = false;
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
      canAutomate = true;
    };

    tls = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-tls;
        release = false;
        injectPhdrs = true;
      };
      isSupported = haveFullRuntime;
      canAutomate = true;
    };

    backtrace = mk {
      rootTask = mkTask {
        rootCrate = crates.tests-root-task-backtrace;
        release = false;
        rootTaskStackSize = 4096 * 64; # TODO
        injectPhdrs = true;
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
                canAutomate = panicStrategyName == "unwind";
              }));

  };

}
