{ mk, localCrates, versions }:

mk {
  package.name = "meta";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-config
    sel4-sync
    sel4-logging
    sel4-sys
    sel4-root-task
    sel4cp
    sel4cp-postcard
  ];
  dependencies = {
    inherit (versions) cfg-if log;
    sel4-root-task = { features = [ "full" ]; optional = true; };
    sel4cp = { features = [ "full" ]; optional = true; };
    sel4cp-postcard = { optional = true; };
  };
  nix.local.target."cfg(not(target_thread_local))".dependencies = with localCrates; [
    sel4
  ];
  target."cfg(not(target_thread_local))".dependencies = {
    sel4 = { features = [ "single-threaded" ]; };
  };
  nix.local.target."cfg(not(target_arch = \"x86_64\"))".dependencies = with localCrates; [
    sel4-platform-info
  ];
  target."cfg(not(target_arch = \"x86_64\"))".dependencies = {
    sel4-platform-info = { optional = true; };
  };
  features = {
    sel4-root-task = [
      "dep:sel4-root-task"
    ];
    sel4cp = [
      "dep:sel4cp"
      "sel4cp-postcard"
    ];
  };
  nix.meta.requirements = [ "sel4" ];
}
