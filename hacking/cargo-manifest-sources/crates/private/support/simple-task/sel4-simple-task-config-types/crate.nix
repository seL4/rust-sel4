{ mk, localCrates, serdeWith, versions }:

mk {
  package.name = "sel4-simple-task-config-types";
  nix.local.target."cfg(target_env = \"sel4\")".dependencies = with localCrates; [
    sel4
    sel4-simple-task-threading
  ];
  dependencies = {
    inherit (versions) cfg-if;
    serde = serdeWith [ "derive" ];
  };
}
