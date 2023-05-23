{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  package.name = "banscii-assistant-core";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
  ];
  dependencies = {
    rusttype = { version = "0.9.3"; default-features = false; features = [ "has-atomics" "libm-math" ]; };
    inherit (versions) log;
    libm = {
      version = "0.2.1";
      default-features = false;
    };
  };
  nix.meta.skip = true;
}
