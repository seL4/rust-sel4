{ mk, versions }:

mk {
  package.name = "sel4-render-elf-with-data";
  dependencies = {
    object = { version = versions.object; features = [ "all" ]; };
    inherit (versions) anyhow fallible-iterator num;
  };
}
