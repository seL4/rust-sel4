{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-simple-task-threading";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-panicking
  ];
  features = {
    alloc = [];
  };
}
