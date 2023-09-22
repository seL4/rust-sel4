{ mk, localCrates, versions }:

mk {
  package.name = "sel4-microkit";
  dependencies = {
    inherit (versions) cfg-if;
    sel4-runtime-common.features = [ "tls" "unwinding" "start" "static-heap" ];
    sel4.features = [ "single-threaded" ];
  };
  features = {
    default = [
      "unwinding"
    ];
    full = [
      "default"
      "alloc"
    ];
    unwinding = [
      "sel4-panicking/unwinding"
    ];
    alloc = [
      "sel4-panicking/alloc"
    ];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-panicking
    sel4-panicking-env
    sel4-runtime-common
    sel4-immediate-sync-once-cell
    sel4-immutable-cell
    sel4-microkit-macros
    sel4-externally-shared
  ];
}
