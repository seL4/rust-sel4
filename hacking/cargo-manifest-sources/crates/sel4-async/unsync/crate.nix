{ mk, versions }:

mk {
  package.name = "sel4-async-unsync";
  dependencies = {
    async-unsync = { version = "0.2.2"; default-features = false; };
  };
}
