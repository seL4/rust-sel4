{ mk, localCrates, serdeWith, coreLicense, meAsAuthor, versions }:

mk {
  package.name = "sel4-simple-task-runtime-config-types";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
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
