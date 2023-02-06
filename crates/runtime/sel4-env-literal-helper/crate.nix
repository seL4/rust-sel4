{ mkLinux }:

mkLinux {
  nix.name = "sel4-env-literal-helper";
  lib.proc-macro = true;
  dependencies = {
    proc-macro2 = "1";
    quote = "1";
    syn = { version = "1"; features = [ "full" ]; };
  };
}
