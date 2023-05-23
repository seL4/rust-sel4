{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-dlmalloc";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4-sync
  ];
  dependencies = {
    dlmalloc = "0.2.3";
  };
}
