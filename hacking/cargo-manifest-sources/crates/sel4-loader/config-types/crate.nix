{ mk, serdeWith, versions }:

mk {
  package.name = "sel4-loader-config-types";
  dependencies = {
    serde = serdeWith [ "derive" ];
  };
}
