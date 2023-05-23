{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-logging";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    inherit (versions) log;
  };
}
