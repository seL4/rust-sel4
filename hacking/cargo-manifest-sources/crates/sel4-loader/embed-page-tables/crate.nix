{ mk, localCrates, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-loader-embed-page-tables";
  dependencies = {
    inherit (versions) proc-macro2;
    inherit (versions) quote;
  };
}
