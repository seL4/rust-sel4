{ lib, hostPlatform, buildPackages
, writeScript, linkFarm
, mkLoader
, worldConfig
, seL4ForBoot

, mkCapDLLoader
, mkSmallCapDLLoader
, mkSimpleCompositionCapDLSpec
}:

let
  defaultTimeout = 30;

  mkInstance = { rootTask, isSupported, canAutomate ? false, automateTimeout ? defaultTimeout, extraLinks ? [] }:
    let

      loader = mkLoader {
        appELF = rootTask.elf;
      };

      instanceForPlatform = worldConfig.mkInstanceForPlatform ({
        rootTask = rootTask.elf;
        loader = loader.elf;
      } // lib.optionalAttrs canAutomate {
        simpleAutomationParams.timeout = automateTimeout;
      });

    in rec {
      inherit rootTask isSupported canAutomate;
      inherit loader instanceForPlatform;

      symbolizeRootTaskBacktrace = writeScript "x.sh" ''
        #!${buildPackages.runtimeShell}
        exec ${buildPackages.this.sel4-symbolize-backtrace}/bin/sel4-symbolize-backtrace -f ${rootTask.elf} "$@"
      '';

      links = linkFarm "links" (
        instanceForPlatform.links
        ++ lib.optionals worldConfig.platformRequiresLoader [
        { name = "loader.elf"; path = loader.elf; }
      ] ++ [
        { name = "kernel.elf"; path = "${seL4ForBoot}/bin/kernel.elf"; }
        { name = "root-task.elf"; path = rootTask.elf; }
        { name = "symbolize-root-task-backtrace"; path = symbolizeRootTaskBacktrace; }
      ] ++ extraLinks ++ (rootTask.extraLinks or []));

    } // instanceForPlatform.attrs;

  mkCorePlatformInstance = { system, extraLinks ? [] }:
    let

      inherit (system) loader;

      instanceForPlatform = worldConfig.mkInstanceForPlatform {
        inherit loader;
      };

    in rec {
      inherit system;

      links = linkFarm "links" (
        instanceForPlatform.links ++ extraLinks ++ system.links);

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
          { name = "x.elf"; path = self.loader.split.full; }
        ];
      }
      // (if small then {
        loader = mkSmallCapDLLoader spec.cdl;
        elf = loader.elf;
      } else {
        loader = mkCapDLLoader spec.cdl;
        elf = loader;
      })
      // passthru
    );
in {
  inherit mkInstance mkCorePlatformInstance mkCapDLRootTask;
}
