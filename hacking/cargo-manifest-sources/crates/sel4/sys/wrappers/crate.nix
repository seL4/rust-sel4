{ mk, localCrates }:

mk {
  package.name = "sel4-sys-wrappers";
  dependencies = {
    sel4-sys.features = [ "wrappers" ];
  };
  lib.crate-type = [ "staticlib" ];
  nix.local.dependencies = with localCrates; [
    sel4-sys
  ];
  nix.meta.requirements = [ "sel4" ];
  nix.meta.skip = true; # HACK
}
