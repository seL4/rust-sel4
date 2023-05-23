{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-loader-embed-page-tables";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    inherit (versions) proc-macro2;
    inherit (versions) quote;
  };
}
