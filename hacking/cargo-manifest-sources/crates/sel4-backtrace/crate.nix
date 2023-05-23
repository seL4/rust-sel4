{ mk, localCrates, postcardWith, unwindingWith, serdeWith, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-backtrace";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-types
    # unwinding # XXX
  ];
  dependencies = {
    inherit (versions) cfg-if;
    unwinding = unwindingWith [] // { optional = true; };
    postcard = postcardWith [] // { optional = true; };
    serde = serdeWith [] // { optional = true; };
  };
  features = {
    alloc = [
      "sel4-backtrace-types/alloc"
    ];
    postcard = [
      "sel4-backtrace-types/postcard"
      "dep:postcard"
      "dep:serde"
    ];
    unwinding = [
      "dep:unwinding"
    ];
    full = [
      "alloc"
      "postcard"
      "unwinding"
    ];
  };
}
