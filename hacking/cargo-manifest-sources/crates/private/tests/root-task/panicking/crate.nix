{ mk, localCrates, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "tests-root-task-panicking";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task-runtime
  ];
  dependencies = {
    inherit (versions) cfg-if;
  };
  features = {
    alloc = [
      "sel4-root-task-runtime/alloc"
    ];
    panic-unwind = [
    ];
    panic-abort = [
    ];
  };
}
