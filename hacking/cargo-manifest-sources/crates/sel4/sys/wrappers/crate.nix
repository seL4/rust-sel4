{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-sys-wrappers";
  nix.local.dependencies = with localCrates; [
    sel4-sys
  ];
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    sel4-sys.features = [ "wrappers" ];
  };
  lib.crate-type = [ "staticlib" ];
  # HACK
  nix.meta.skip = true;
}
