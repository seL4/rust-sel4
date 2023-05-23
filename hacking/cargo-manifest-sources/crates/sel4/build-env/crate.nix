{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-build-env";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
}
