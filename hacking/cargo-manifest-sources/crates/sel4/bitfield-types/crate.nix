{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-bitfield-types";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
}
