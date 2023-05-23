{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-sys-wrappers";
  nix.local.dependencies = with localCrates; [
    sel4-sys
  ];
  dependencies = {
    sel4-sys.features = [ "wrappers" ];
  };
  lib.crate-type = [ "staticlib" ];
  # HACK
  nix.meta.skip = true;
}
