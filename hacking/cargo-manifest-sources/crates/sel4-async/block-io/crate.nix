{ mk, versions }:

mk {
  package.name = "sel4-async-block-io";
  dependencies = rec {
    inherit (versions) log;
    lru = { version = "0.10.0"; optional = true; };
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
      optional = true;
    };
  };
  features = {
    alloc = [ "lru" "futures" ];
    default = [ "alloc" ];
  };
}
