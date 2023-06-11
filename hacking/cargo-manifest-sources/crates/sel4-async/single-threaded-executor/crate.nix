{ mk, versions }:

mk {
  package.name = "sel4-async-single-threaded-executor";
  package.license = "MIT OR Apache-2.0";
  dependencies = {
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
  };
}
