{ mk, localCrates, versions, smoltcpWith, mbedtlsWith }:

mk {
  package.name = "tests-capdl-http-server-components-test-server-core";
  dependencies = {
    inherit (versions) log;

    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "async-await"
        "alloc"
      ];
    };

    httparse = { version = "1.8.0"; default-features = false; };

    smoltcp = smoltcpWith [];

    mbedtls = mbedtlsWith [];
  };
  nix.local.dependencies = with localCrates; [
    sel4-async-single-threaded-executor
    sel4-async-network
    sel4-async-network-mbedtls
    sel4-async-timers
    sel4-panicking-env
    tests-capdl-http-server-components-test-cpiofs
    mbedtls
  ];
  features = {
    debug = [ "mbedtls/debug" ];
  };
}
