{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "sel4-simple-task-config-types";
  dependencies = {
    serde = serdeWith [ "derive" ];
    inherit (versions) cfg-if;
  };
  nix.local.target."cfg(target_env = \"sel4\")".dependencies = with localCrates; [
    sel4
    sel4-simple-task-threading
  ];
}
