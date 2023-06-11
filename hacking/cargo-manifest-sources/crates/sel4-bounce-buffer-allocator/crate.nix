{ mk, versions }:

mk {
  package.name = "sel4-bounce-buffer-allocator";
  dependencies = {
    inherit (versions) log;
  };
}
