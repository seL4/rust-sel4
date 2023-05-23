{ mk, localCrates, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4cp";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-panicking
    sel4-panicking-env
    sel4-runtime-common
    sel4-immediate-sync-once-cell
    sel4cp-macros
    sel4-externally-shared
  ];
  dependencies = {
    inherit (versions) cfg-if;
    num_enum = { version = versions.num_enum; default-features = false; };
    inherit (versions) zerocopy;
    sel4-runtime-common.features = [ "tls" "unwinding" "start" "static-heap" ];
    sel4.features = [ "single-threaded" ];
    sel4-externally-shared.features = [ "unstable" ];
  };
  features = {
    default = [
      "unwinding"
    ];
    full = [
      "default"
      "alloc"
    ];
    unwinding = [
      "sel4-panicking/unwinding"
    ];
    alloc = [
      "zerocopy/alloc"
      "sel4-externally-shared/alloc"
      "sel4-panicking/alloc"
    ];
  };
  nix.meta.skip = true;
}
