{ mk, localCrates, versions, unwindingWith }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-runtime-common";
  dependencies = {
    inherit (versions) cfg-if;
    unwinding = unwindingWith [] // { optional = true; };
    sel4-reserve-tls-on-stack.optional = true;
    sel4-sync.optional = true;
    sel4-dlmalloc.optional = true;
  };
  nix.local.dependencies = with localCrates; [
    sel4-reserve-tls-on-stack
    sel4-sync
    sel4-dlmalloc
  ];
  features = {
    tls = [ "dep:sel4-reserve-tls-on-stack" ];
    start = [];
    static-heap = [ "sel4-sync" "sel4-dlmalloc" ];
  };
}
