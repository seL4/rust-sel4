{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-rustfmt-helper";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    which = "4.3.0";
  };
}
