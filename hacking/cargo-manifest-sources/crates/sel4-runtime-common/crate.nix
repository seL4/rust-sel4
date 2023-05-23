{ mk, localCrates, versions, unwindingWith }:

mk {
  package.name = "sel4-runtime-common";
  dependencies = {
    inherit (versions) cfg-if;
    unwinding = unwindingWith [] // { optional = true; };
    sel4-reserve-tls-on-stack.optional = true;
    sel4-sync.optional = true;
    sel4-dlmalloc.optional = true;
  };
  features = {
    tls = [ "dep:sel4-reserve-tls-on-stack" ];
    start = [];
    static-heap = [ "sel4-sync" "sel4-dlmalloc" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-reserve-tls-on-stack
    sel4-sync
    sel4-dlmalloc
  ];

  nix.meta.requirements = [ "sel4" ];
}
