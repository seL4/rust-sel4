{ mk, localCrates, versions  }:

mk {
  package.name = "sel4-backtrace-embedded-debug-info-cli";
  dependencies = {
    inherit (versions) num clap;
  };
  nix.local.dependencies = with localCrates; [
    sel4-render-elf-with-data
  ];
}
