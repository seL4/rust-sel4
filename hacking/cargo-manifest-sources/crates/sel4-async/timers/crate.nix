{ mk, versions, smoltcpWith }:

mk {
  package.name = "sel4-async-timers";
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
