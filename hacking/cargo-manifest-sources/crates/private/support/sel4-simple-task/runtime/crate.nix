{ mk, localCrates, serdeWith, postcardWith, versions }:

mk {
  package.name = "sel4-simple-task-runtime";
  dependencies = {
    sel4-backtrace.features = [ "unwinding" "postcard" ];
    sel4-runtime-common.features = [ "tls" "unwinding" ];
    serde = serdeWith [];
    postcard = postcardWith [];
    serde_json = { version = versions.serde_json; default-features = false; optional = true; };
  };
  features = {
    serde_json = [
      "dep:serde_json"
    ];
    alloc = [
      "sel4-backtrace/alloc"
      "sel4-backtrace-simple/alloc"
      "sel4-simple-task-threading/alloc"
      "serde_json?/alloc"
    ];
    default = [
      "serde_json"
      "alloc"
    ];
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-backtrace
    sel4-backtrace-simple
    sel4-dlmalloc
    sel4-immediate-sync-once-cell
    sel4-panicking
    sel4-panicking-env
    sel4-reserve-tls-on-stack
    sel4-runtime-common
    sel4-simple-task-runtime-config-types
    sel4-simple-task-runtime-macros
    sel4-simple-task-threading
    sel4-sync
  ];
  nix.meta.requirements = [ "sel4" ];
}
