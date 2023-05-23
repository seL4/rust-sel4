{ mk, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-simple-task-runtime-macros";
  lib.proc-macro = true;
  dependencies = {
    inherit (versions) proc-macro2;
    inherit (versions) quote;
    syn = { version = versions.syn; features = [ "full" ]; };
  };
}
