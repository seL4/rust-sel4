{ mk, localCrates }:

mk {
  package.name = "sel4-backtrace-simple";
  dependencies = {
    sel4-backtrace.features = [ "postcard" "unwinding" ];
  };
  features = {
    alloc = [
      "sel4-backtrace/alloc"
    ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-backtrace
    sel4-panicking-env
  ];
  nix.meta.requirements = [ "sel4" ];
}
