{ mk, versions }:

mk {
  package.name = "capdl-initializer-types-derive";
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
