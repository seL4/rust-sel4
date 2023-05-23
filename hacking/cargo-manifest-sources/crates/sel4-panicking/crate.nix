{ mk, localCrates, versions, unwindingWith }:

mk {
  package.name = "sel4-panicking";
  dependencies = {
    inherit (versions) cfg-if;
    unwinding = unwindingWith [ "personality" ] // { optional = true; };
  };
  features = {
    alloc = [];
  };
  nix.local.dependencies = with localCrates; [
    sel4-panicking-env
    sel4-immediate-sync-once-cell
  ];
  nix.meta.requirements = [ "sel4" ];
}
