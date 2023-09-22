{ mk, localCrates }:

mk {
  package.name = "tests-root-task-c";
  build-dependencies = {
    cc = "1.0.76";
    glob = "0.3.0";
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task
    sel4-async-network-mbedtls
    sel4-newlib
  ];
}
