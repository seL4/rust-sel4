{ mk, serdeWith, coreLicense, meAsAuthor, versions }:

mk {
  package.name = "sel4-loader-config-types";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    serde = serdeWith [ "derive" ];
  };
}
