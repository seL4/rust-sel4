{ mk, versions }:

mk {
  package.name = "sel4-backtrace-addr2line-context-helper";
  dependencies = {
    addr2line = { version = versions.addr2line; default-features = false; features = [ "rustc-demangle" "cpp_demangle" "fallible-iterator" "smallvec" "object" ]; };
    gimli = { version = versions.gimli; default-features = false; features = [ "endian-reader" ]; };
    stable_deref_trait = { version = "1.1.0"; default-features = false; features = [ "alloc" ]; };
  };
}
