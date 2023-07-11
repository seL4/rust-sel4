{ mk, localCrates, versions }:

mk {
  package.name = "sel4-simple-task-serialize-runtime-config";
  dependencies = {
    sel4-simple-task-runtime-config-types = { features = [ "serde" "alloc" ]; };
    inherit (versions) serde_json;
  };
  nix.local.dependencies = with localCrates; [
    sel4-simple-task-runtime-config-types
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
}
