{ mk, versions, smoltcpWith }:

mk {
  package.name = "sel4-async-network";
  dependencies = {
    inherit (versions) log;
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
    smoltcp = smoltcpWith [];
  };
}
