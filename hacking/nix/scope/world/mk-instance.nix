#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, hostPlatform, buildPackages
, writeScript, linkFarm
, mkSeL4KernelWithPayload
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

      loader = mkSeL4KernelWithPayload {
        appELF = rootTask.elf;
      };

      instanceForPlatform = worldConfig.mkInstanceForPlatform ({
        rootTask = rootTask.elf;
        loader = loader.elf;
      } // extraPlatformArgs);

    in rec {
      inherit loader rootTask instanceForPlatform;

      symbolizeRootTaskBacktrace = writeScript "x.sh" ''
        #!${buildPackages.runtimeShell}
        exec ${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace -f ${rootTask.elf} "$@"
      '';

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

  mkCapDLRootTask =
    { script
    , config
    , passthru
    , small ? false
    }:
    let
    in lib.fix (self: with self;
      {
        inherit script config;
        spec = mkSimpleCompositionCapDLSpec {
          inherit script config;
        };
        extraLinks = [
          { name = "cdl"; path = spec; }
        ] ++ lib.optionals (!small) [
          { name = "initializer.full.elf"; path = self.initializer.split.full; }
        ];
      }
      // (if small then {
        initializer = mkSmallCapDLInitializer spec.cdl;
        elf = initializer.elf;
      } else {
        initializer = mkCapDLInitializer spec.cdl;
        elf = initializer.elf;
      })
      // passthru
    );
in {
  inherit mkInstance mkMicrokitInstance mkCapDLRootTask;
}
