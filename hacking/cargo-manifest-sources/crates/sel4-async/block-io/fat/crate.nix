{ mk, localCrates, versions, fatSource }:

mk {
  package.name = "sel4-async-block-io-fat";
  dependencies = rec {
    inherit (versions) log heapless;
    hex = { version = "0.4.3"; default-features = false; };
    lru = "0.10.0";
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
    embedded-fat = fatSource;
  };
  nix.local.dependencies = with localCrates; [
    sel4-async-block-io
    # embedded-fat
  ];
}
