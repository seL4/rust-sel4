{ mk, localCrates, unwindingWith, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-panicking";
  nix.local.dependencies = with localCrates; [
    sel4-panicking-env
    sel4-immediate-sync-once-cell
    # unwinding # XXX
  ];
  dependencies = {
    inherit (versions) cfg-if;
    unwinding = unwindingWith [ "personality" ] // { optional = true; };
  };
  features = {
    alloc = [];
  };
}
