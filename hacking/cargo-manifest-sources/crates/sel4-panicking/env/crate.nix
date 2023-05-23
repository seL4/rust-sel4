{ mk, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-panicking-env";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
}
