{ mk, localCrates, serdeWith, postcardWith }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-simple-task-rpc";
  nix.local.dependencies = with localCrates; [
    sel4
  ];
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
}
