{ mk, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "sel4-render-elf-with-data";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    object = { version = versions.object; features = [ "all" ]; };
    clap = "3.2.23";
    inherit (versions) anyhow;
    inherit (versions) fallible-iterator;
  };
}
