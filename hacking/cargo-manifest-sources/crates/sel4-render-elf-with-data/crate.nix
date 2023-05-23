{ mk, versions }:

mk {
  package.name = "sel4-render-elf-with-data";
  dependencies = {
    object = { version = versions.object; features = [ "all" ]; };
    clap = "3.2.23";
    inherit (versions) anyhow fallible-iterator;
  };
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
