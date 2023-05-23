{ mk, serdeWith, coreLicense, meAsAuthor, versions }:

mk {
  package.name = "sel4-loader-payload-types";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    serde = serdeWith [ "derive" ] // { optional = true; };
    heapless = { version = versions.heapless; features = [ "serde" ]; };
  };
}
