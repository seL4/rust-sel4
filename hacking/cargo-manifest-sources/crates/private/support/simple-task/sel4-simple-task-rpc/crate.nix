{ mk, localCrates, serdeWith, postcardWith, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-simple-task-rpc";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
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
