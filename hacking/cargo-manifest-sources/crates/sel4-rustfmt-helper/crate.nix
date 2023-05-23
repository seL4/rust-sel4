{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-rustfmt-helper";
  dependencies = {
    which = "4.3.0";
  };
}
