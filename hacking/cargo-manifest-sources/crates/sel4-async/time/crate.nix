{ mk, versions, smoltcpWith }:

mk {
  package.name = "sel4-async-time";
  dependencies = {
    inherit (versions) log pin-project;
    # futures = {
    #   version = versions.futures;
    #   default-features = false;
    #   features = [
    #     "alloc"
    #   ];
    # };
  };
}
