{ mk, versions, serdeWith }:

mk {
  package.name = "sel4-simple-task-runtime-config-types";
  dependencies = {
    inherit (versions) zerocopy;
    serde = serdeWith [ "derive" ] // { optional = true; };
  };
  features = {
    alloc = [
      "serde?/alloc"
    ];
    serde = [
      "dep:serde"
    ];
  };
}
