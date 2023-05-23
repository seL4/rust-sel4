{ mk, localCrates, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  package.name = "banscii-assistant-core-test";
  nix.local.dependencies = with localCrates; [
    banscii-assistant-core
  ];
  dependencies = {
    env_logger = "0.10.0";
    inherit (versions) log;
  };
  nix.meta.skip = true;
}
