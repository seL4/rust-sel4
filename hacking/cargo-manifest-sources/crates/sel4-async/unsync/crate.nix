{ mk, versions }:

mk {
  package.name = "sel4-async-unsync";
  dependencies = {
    async-unsync = { version = versions.async-unsync; default-features = false; };
  };
}
