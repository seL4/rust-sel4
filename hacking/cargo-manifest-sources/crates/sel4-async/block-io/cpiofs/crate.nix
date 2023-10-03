{ mk, localCrates, versions, zerocopyWith }:

mk {
  package.name = "sel4-async-block-io-cpiofs";
  dependencies = rec {
    inherit (versions) log;
    zerocopy = zerocopyWith [ "derive" ];
    hex = { version = "0.4.3"; default-features = false; };
    lru = "0.10.0";
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
  };
  nix.local.dependencies = with localCrates; [
    sel4-async-block-io
  ];
}
