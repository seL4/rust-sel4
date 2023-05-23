{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "sel4-simple-task-serialize-runtime-config";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4-simple-task-runtime-config-types
  ];
  dependencies = {
    sel4-simple-task-runtime-config-types = { features = [ "serde" "alloc" ]; };
    inherit (versions) serde_json;
  };
}
