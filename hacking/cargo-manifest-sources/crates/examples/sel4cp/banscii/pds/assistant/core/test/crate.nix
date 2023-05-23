{ mk, localCrates, versions }:

mk {
  package.name = "banscii-assistant-core-test";
  dependencies = {
    env_logger = "0.10.0";
    inherit (versions) log;
  };
  nix.local.dependencies = with localCrates; [
    banscii-assistant-core
  ];
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "unix" ];
  nix.meta.skip = true;
}
