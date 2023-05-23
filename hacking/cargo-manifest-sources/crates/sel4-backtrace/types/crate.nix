{ mk, serdeWith, postcardWith, coreLicense, meAsAuthor, versions }:

mk {
  package.name = "sel4-backtrace-types";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    inherit (versions) cfg-if;
    serde = serdeWith [ "derive" ] // { optional = true; };
    postcard = postcardWith [] // { optional = true; };
    addr2line = { version = "0.20.0"; default-features = false; features = [ "rustc-demangle" "cpp_demangle" "object" "fallible-iterator" "smallvec" ]; optional = true; };
    fallible-iterator = { version = versions.fallible-iterator; default-features = false; optional = true; };
  };
  features = {
    alloc = [
      "serde?/alloc"
    ];
    serde = [
      "dep:serde"
    ];
    postcard = [
      "serde"
      "dep:postcard"
    ];
    symbolize = [
      "addr2line"
      "fallible-iterator"
      "alloc"
    ];
    full = [
      "alloc"
      "serde"
      "postcard"
      "symbolize"
    ];
  };
}
