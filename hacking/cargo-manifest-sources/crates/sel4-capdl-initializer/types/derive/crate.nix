{ mk, versions }:

mk {
  package.name = "sel4-capdl-initializer-types-derive";
  lib.proc-macro = true;
  dependencies = {
    inherit (versions)
      proc-macro2
      quote
      syn
      synstructure
    ;
  };
}
