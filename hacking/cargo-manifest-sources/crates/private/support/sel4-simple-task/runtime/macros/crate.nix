{ mk, versions }:

mk {
  package.name = "sel4-simple-task-runtime-macros";
  lib.proc-macro = true;
  dependencies = {
    inherit (versions) proc-macro2 quote;
    syn = { version = versions.syn; features = [ "full" ]; };
  };
}
