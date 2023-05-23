{ mk, serdeWith }:

mk {
  package.name = "sel4-config-generic-types";
  dependencies = {
    serde = serdeWith [ "alloc" "derive" ] // { optional = true; };
  };
}
