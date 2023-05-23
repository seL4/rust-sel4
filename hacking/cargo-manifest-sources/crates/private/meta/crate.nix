{ mk, localCrates, versions }:

mk {
  package.name = "meta";
  nix.local.dependencies = with localCrates; [
    capdl-types
  ];
  dependencies = {
    inherit (versions) cfg-if log;
    capdl-types = { features = [ "alloc" ]; };
  };
  nix.local.target."cfg(not(target_os = \"none\"))".dependencies = with localCrates; [
    sel4-render-elf-with-data
  ];
  target."cfg(not(target_os = \"none\"))".dependencies = {
  };
  nix.local.target."cfg(target_env = \"sel4\")".dependencies = with localCrates; [
    sel4
    sel4-config
    sel4-sync
    sel4-logging
    sel4-sys
    sel4-root-task-runtime
    sel4-backtrace
    sel4-backtrace-types
    sel4-backtrace-embedded-debug-info
    sel4-backtrace-simple
    sel4cp
    sel4cp-postcard
    # capdl-types
  ];
  target."cfg(target_env = \"sel4\")".dependencies = {
    sel4-root-task-runtime = { features = [ "full" ]; optional = true; };
    sel4cp = { features = [ "full" ]; optional = true; };
    sel4cp-postcard = { optional = true; };
    sel4-backtrace = { features = [ "full" ]; };
    sel4-backtrace-types = { features = [ "full" ]; };
    sel4-backtrace-simple = { features = [ "alloc" ]; };
    # capdl-types = { features = [ "sel4" "alloc" ]; };
  };
  # nix.local.target."cfg(not(target_env = \"sel4\"))".dependencies = with localCrates; [
  #   capdl-types
  # ];
  # target."cfg(not(target_env = \"sel4\"))".dependencies = {
  #   capdl-types = { features = [ "alloc" ]; };
  # };
  nix.local.target."cfg(all(target_env = \"sel4\", not(target_arch = \"x86_64\")))".dependencies = with localCrates; [
    sel4-platform-info
  ];
  target."cfg(all(target_env = \"sel4\", not(target_arch = \"x86_64\")))".dependencies = {
    sel4-platform-info = { optional = true; };
  };
  features = {
    root-task = [
      "sel4-root-task-runtime"
    ];
    sel4cp = [
      "dep:sel4cp"
      "sel4cp-postcard"
    ];
  };
  nix.meta.requirements = [ "sel4" ];
}
