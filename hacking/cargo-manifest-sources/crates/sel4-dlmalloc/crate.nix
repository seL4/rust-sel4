{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-dlmalloc";
  nix.local.dependencies = with localCrates; [
    sel4-sync
  ];
  dependencies = {
    dlmalloc = "0.2.3";
  };
}
