{ mk, versions }:

mk {
  package.name = "banscii-assistant-core";
  dependencies = {
    rusttype = { version = "0.9.3"; default-features = false; features = [ "has-atomics" "libm-math" ]; };
    libm = { version = "0.2.1"; default-features = false; };
    inherit (versions) log;
  };
  nix.meta.skip = true;
}
