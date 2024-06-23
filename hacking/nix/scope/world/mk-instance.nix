#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, hostPlatform, buildPackages
, writeScript, linkFarm
, mkSeL4KernelLoaderWithPayload
, worldConfig
, seL4ForBoot

, mkCapDLInitializer
, mkSmallCapDLInitializer
, mkSimpleCompositionCapDLSpec
}:

let
  defaultTimeout = 30;

  symbolizeBacktraceLinksEntry = {
    name = "sel4-symbolize-backtrace";
    path = "${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace";
  };

  mkInstance =
    { rootTask
    , extraLinks ? []
    , extraPlatformArgs ? {}
    }:

    let
      loader = mkSeL4KernelLoaderWithPayload {
        appELF = rootTask.elf;
      };

      instanceForPlatform = worldConfig.mkInstanceForPlatform ({
        rootTask = rootTask.elf;
        loader = loader.elf;
      } // extraPlatformArgs);

      symbolizeRootTaskBacktrace = writeScript "x.sh" ''
        #!${buildPackages.runtimeShell}
        exec ${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace -f ${rootTask.elf} "$@"
      '';

    in rec {
      inherit loader rootTask instanceForPlatform;

      links = linkFarm "links" (lib.concatLists [
        instanceForPlatform.links
        (lib.optionals worldConfig.platformRequiresLoader [
          { name = "loader.elf"; path = loader.elf; }
          { name = "loader.debug.elf"; path = loader.split.full; }
        ])
        [
          symbolizeBacktraceLinksEntry
          { name = "kernel.elf"; path = "${seL4ForBoot}/bin/kernel.elf"; }
          { name = "root-task.elf"; path = rootTask.elf; }
          { name = "symbolize-root-task-backtrace"; path = symbolizeRootTaskBacktrace; }
        ]
        extraLinks
        (rootTask.extraLinks or [])
      ]);

    } // instanceForPlatform.attrs;

  mkMicrokitInstance =
    { system
    , extraLinks ? []
    , extraPlatformArgs ? {}
    }:

    let
      instanceForPlatform = worldConfig.mkInstanceForPlatform ({
        inherit (system) loader;
      } // extraPlatformArgs);

    in rec {
      inherit system instanceForPlatform;

      links = linkFarm "links" (lib.concatLists [
        [
          symbolizeBacktraceLinksEntry
        ]
        instanceForPlatform.links
        extraLinks
        system.links
      ]);

    } // instanceForPlatform.attrs;

  mkSimpleCompositionCapDLRootTask =
    { spec ? mkSimpleCompositionCapDLSpec (builtins.removeAttrs args [
        "small" "passthru" "extraLinks" "spec"
      ])
    , small ? false
    , extraLinks ? []
    , ...
    } @ args:

    lib.fix (self: with self;
      (if small then mkSmallCapDLInitializer else mkCapDLInitializer) spec.cdl // {
        specDrv = spec; # HACK
        extraLinks = [
          { name = "cdl"; path = spec; }
        ] ++ lib.optionals (!small) [
          { name = "initializer.full.elf"; path = self.split.full; }
        ] ++ extraLinks;
      }
    );

in {
  inherit mkInstance mkMicrokitInstance mkSimpleCompositionCapDLRootTask;
}
