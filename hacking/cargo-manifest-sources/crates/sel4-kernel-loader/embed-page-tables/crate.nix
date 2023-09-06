{ mk, versions }:

mk {
  package.name = "sel4-kernel-loader-embed-page-tables";
  dependencies = {
    inherit (versions) proc-macro2 quote;
    bitfield = "0.14";
  };
  nix.meta.requirements = [ "linux" ];
}
