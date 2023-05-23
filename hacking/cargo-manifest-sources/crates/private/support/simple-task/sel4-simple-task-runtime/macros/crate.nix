{ mk, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "linux" ];
  package.name = "sel4-simple-task-runtime-macros";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  lib.proc-macro = true;
  dependencies = {
    inherit (versions) proc-macro2;
    inherit (versions) quote;
    syn = { version = versions.syn; features = [ "full" ]; };
  };
}
