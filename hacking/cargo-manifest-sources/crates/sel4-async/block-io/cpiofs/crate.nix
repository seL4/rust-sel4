{ mk, versions }:

mk {
  package.name = "sel4-async-block-io-cpiofs";
  dependencies = rec {
    inherit (versions) log zerocopy;
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
}
