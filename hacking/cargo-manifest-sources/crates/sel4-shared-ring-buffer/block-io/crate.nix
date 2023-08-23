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

    async-unsync = { version = "0.2.2"; default-features = false; };

    sel4-externally-shared.features = [ "unstable" ];
  };
  nix.local.dependencies = with localCrates; [
    sel4-externally-shared
    sel4-shared-ring-buffer
    sel4-shared-ring-buffer-block-io-types
    sel4-bounce-buffer-allocator
    sel4-async-request-statuses
    sel4-async-block-io
  ];
}
