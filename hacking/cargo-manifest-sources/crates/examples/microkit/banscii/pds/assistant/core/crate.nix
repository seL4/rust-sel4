{ mk, versions }:

mk {
  package.name = "banscii-assistant-core";
  dependencies = {
    ab_glyph = { version = "0.2.22"; default-features = false; features = [ "libm" ]; };
    num-traits = { version = versions.num-traits; default-features = false; features = [ "libm" ]; };
    inherit (versions) log;
  };
}
