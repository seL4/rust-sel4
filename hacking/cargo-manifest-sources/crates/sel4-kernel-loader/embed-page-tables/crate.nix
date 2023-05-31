{ mk, versions }:

mk {
  package.name = "sel4-kernel-loader-embed-page-tables";
  dependencies = {
    inherit (versions) proc-macro2 quote;
  };
  nix.meta.requirements = [ "linux" ];
}
