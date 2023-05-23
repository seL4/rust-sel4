{ mk, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "capdl-types-derive";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  lib.proc-macro = true;
  dependencies = {
    inherit (versions) proc-macro2;
    inherit (versions) quote;
    inherit (versions) syn;
    inherit (versions) synstructure;
  };
}
