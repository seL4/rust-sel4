{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-backtrace-simple";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4-backtrace
    sel4-panicking-env
  ];
  dependencies = {
    sel4-backtrace.features = [ "postcard" "unwinding" ];
  };
  features = {
    alloc = [
      "sel4-backtrace/alloc"
    ];
  };
}
