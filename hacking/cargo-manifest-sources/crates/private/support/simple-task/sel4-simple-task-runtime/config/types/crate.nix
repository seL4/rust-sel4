{ mk, localCrates, serdeWith, versions }:

mk {
  package.name = "sel4-simple-task-runtime-config-types";
  nix.local.dependencies = with localCrates; [
  ];
  dependencies = {
    inherit (versions) zerocopy;
    serde = serdeWith [ "derive" ] // {
      optional = true;
    };
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
