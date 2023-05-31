{ mk, serdeWith }:

mk {
  package.name = "sel4-kernel-loader-config-types";
  dependencies = {
    serde = serdeWith [ "derive" ];
  };
}
