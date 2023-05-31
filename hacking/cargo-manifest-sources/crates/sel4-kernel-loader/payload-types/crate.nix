{ mk, serdeWith, versions }:

mk {
  package.name = "sel4-kernel-loader-payload-types";
  dependencies = {
    serde = serdeWith [ "derive" ] // { optional = true; };
    heapless = { version = versions.heapless; features = [ "serde" ]; };
  };
}
