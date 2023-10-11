{ mk, localCrates, versions }:

mk {
  package.name = "sel4-shared-ring-buffer-block-io";
  dependencies = rec {
    inherit (versions) log;

    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "async-await"
        "alloc"
      ];
    };

    async-unsync = { version = versions.async-unsync; default-features = false; };

    sel4-externally-shared.features = [ "unstable" ];

    sel4-shared-ring-buffer-bookkeeping = { features = [ "async-unsync" ]; };
  };
  nix.local.dependencies = with localCrates; [
    sel4-externally-shared
    sel4-shared-ring-buffer
    sel4-shared-ring-buffer-block-io-types
    sel4-bounce-buffer-allocator
    sel4-shared-ring-buffer-bookkeeping
    sel4-async-block-io
  ];
}
