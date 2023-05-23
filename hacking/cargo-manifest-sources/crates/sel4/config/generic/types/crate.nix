{ mk, serdeWith, coreLicense, meAsAuthor }:

mk {
  package.name = "sel4-config-generic-types";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    serde = serdeWith [ "alloc" "derive" ] // { optional = true; };
  };
}
