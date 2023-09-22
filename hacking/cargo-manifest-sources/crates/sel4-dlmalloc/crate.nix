{ mk, localCrates }:

mk {
  package.name = "sel4-dlmalloc";
  dependencies = {
    dlmalloc = "0.2.3";
  };
  nix.local.dependencies = with localCrates; [
    sel4-sync
  ];
}
