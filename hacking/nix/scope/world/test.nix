{ lib, buildPackages
, writeScript
, crates
, mkTask, mkLoader
, worldConfig
}:

let
  mk = { task }: rec {
    loader = mkLoader {
      appELF = task.elf;
    };

    qemuCmd = worldConfig.mkQemuCmd (if worldConfig.qemuCmdRequiresLoader then loader.elf else task.elf);

    run = writeScript "run.sh" ''
      #!${buildPackages.runtimeShell}

      ${lib.concatStringsSep " " qemuCmd}
    '';
  };

in rec {

  minimal-runtime-with-state = mk {
    task = mkTask {
      rootCrate = crates.minimal-runtime-with-state;
      release = false;
    };
  };

  full-runtime = mk {
    task = mkTask {
      rootCrate = crates.full-runtime;
      release = false;
      injectPhdrs = true;
    };
  };

}
