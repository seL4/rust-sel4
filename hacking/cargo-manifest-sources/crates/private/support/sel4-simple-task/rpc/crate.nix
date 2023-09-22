{ mk, localCrates, serdeWith, postcardWith }:

mk {
  package.name = "sel4-simple-task-rpc";
  dependencies = {
    serde = serdeWith [] // { optional = true; };
    postcard = postcardWith [] // { optional = true; };
  };
  features = {
    postcard = [
      "dep:serde"
      "dep:postcard"
    ];
  };
  nix.local.dependencies = with localCrates; [
    sel4
  ];
}
