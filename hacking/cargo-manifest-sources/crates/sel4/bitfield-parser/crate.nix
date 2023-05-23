{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-bitfield-parser";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = rec {
    regex = "1.7.0";
    pest = "2.4.1";
    pest_derive = pest;
  };
}
