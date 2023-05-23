{ mk, versions }:

mk {
  package.name = "sel4-loader-embed-page-tables";
  dependencies = {
    inherit (versions) proc-macro2 quote;
  };
  nix.meta.requirements = [ "linux" ];
}
