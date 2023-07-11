{ mk, localCrates }:

mk {
  package.name = "sel4-simple-task-threading";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-panicking
  ];
  features = {
    alloc = [];
  };
  nix.meta.requirements = [ "sel4" ];
}
