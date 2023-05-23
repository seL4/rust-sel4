{ mk, localCrates }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-backtrace-simple";
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
