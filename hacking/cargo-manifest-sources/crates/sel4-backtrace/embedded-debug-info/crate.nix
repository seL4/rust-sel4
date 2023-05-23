{ mk, localCrates, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-backtrace-embedded-debug-info";
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-addr2line-context-helper
  ];
  dependencies = {
    addr2line = { version = "0.20.0"; default-features = false; };
    object = { version = versions.object; default-features = false; features = [ "read" ]; };
  };
}
