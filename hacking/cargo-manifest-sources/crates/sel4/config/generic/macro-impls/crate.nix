{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-config-generic-macro-impls";
  nix.local.dependencies = with localCrates; [
    sel4-config-generic-types
  ];
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    inherit (versions) fallible-iterator;
    inherit (versions) proc-macro2;
    inherit (versions) quote;
    syn = { version = versions.syn; features = [ "full" ]; };
  };
}
