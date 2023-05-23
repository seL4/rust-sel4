{ mk, localCrates, serdeWith, postcardWith, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4cp-postcard";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4
    sel4cp
  ];
  dependencies = {
    serde = serdeWith [];
    postcard = postcardWith [];
  };
  nix.meta.skip = true;
}
