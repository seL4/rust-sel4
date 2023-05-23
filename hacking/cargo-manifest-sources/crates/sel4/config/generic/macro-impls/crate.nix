{ mk, localCrates, versions }:

mk {
  package.name = "sel4-config-generic-macro-impls";
  dependencies = {
    syn = { version = versions.syn; features = [ "full" ]; };
    inherit (versions)
      fallible-iterator
      proc-macro2
      quote
    ;
  };
  nix.local.dependencies = with localCrates; [
    sel4-config-generic-types
  ];
  nix.meta.requirements = [ "linux" ];
}
