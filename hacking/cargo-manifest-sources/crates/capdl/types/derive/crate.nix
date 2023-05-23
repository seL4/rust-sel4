{ mk, versions }:

mk {
  package.name = "capdl-types-derive";
  lib.proc-macro = true;
  dependencies = {
    inherit (versions)
      proc-macro2
      quote
      syn
      synstructure
    ;
  };
  nix.meta.requirements = [ "linux" ];
}
