{ mk, versions }:

mk {
  package.name = "banscii-assistant-core";
  dependencies = {
    ab_glyph = { version = "0.2.22"; default-features = false; features = [ "libm" ]; };
    libm = { version = "0.2.1"; default-features = false; };
    inherit (versions) log;
  };
}
