{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "tests-root-task-c";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-root-task-runtime
  ];
  dependencies = {
    # mbedtls = {
    #   # version = "0.9.0";
    #   git = "https://github.com/fortanix/rust-mbedtls";
    #   rev = "07e2cf171b538b188501c1faa9dfdc2b92299ed2";
    #   default-features = false;
    #   features = [ "no_std_deps" ];
    # };
  };
  build-dependencies = {
    cc = "1.0.76";
    glob = "0.3.0";
  };
  nix.meta.skip = true;
}
