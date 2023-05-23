{ mk, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-immediate-sync-once-cell";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
}
