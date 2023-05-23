{ mk, localCrates, postcardCommon, coreLicense, meAsAuthor }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "sel4-embed-debug-info";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4-render-elf-with-data
  ];
  dependencies = {
    clap = "3.2.23";
  };
}
