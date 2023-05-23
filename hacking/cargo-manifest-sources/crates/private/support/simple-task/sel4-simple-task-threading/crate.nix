{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-simple-task-threading";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-panicking
  ];
  features = {
    alloc = [];
  };
}
