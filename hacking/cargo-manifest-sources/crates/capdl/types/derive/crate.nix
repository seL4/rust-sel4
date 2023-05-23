{ mk, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "capdl-types-derive";
  lib.proc-macro = true;
  dependencies = {
    inherit (versions) proc-macro2;
    inherit (versions) quote;
    inherit (versions) syn;
    inherit (versions) synstructure;
  };
}
