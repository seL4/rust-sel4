{ mk, versions }:

mk {
  package.name = "tests-capdl-http-server-components-http-server-cpiofs";
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
