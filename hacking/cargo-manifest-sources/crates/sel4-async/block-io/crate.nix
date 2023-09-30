{ mk, versions }:

mk {
  package.name = "sel4-async-block-io";
  dependencies = rec {
    inherit (versions) log num-derive;
    futures = {
      version = versions.futures;
      default-features = false;
    };
    num-traits = { version = versions.num-traits; default-features = false; };
    bytemuck = { version = "1.4.0"; default-features = false; };
    gpt_disk_types = { version = "0.15.0"; features = [ "bytemuck" ]; };
    lru = { version = "0.10.0"; optional = true; };
  };
  features = {
    alloc = [ "futures/alloc" "lru" ];
    default = [ "alloc" ];
  };
}
