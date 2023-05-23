{ mk, localCrates }:

mk {
  package.name = "sel4-embed-debug-info";
  dependencies = {
    clap = "3.2.23";
  };
  nix.local.dependencies = with localCrates; [
    sel4-render-elf-with-data
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
