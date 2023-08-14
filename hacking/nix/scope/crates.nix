{ lib
, crateUtils
, sources
}:

let
  workspaceDir = sources.srcRoot;

  workspaceManifest = builtins.fromTOML (builtins.readFile workspaceManifestPath);
  workspaceManifestPath = workspaceDir + "/Cargo.toml";
  workspaceMemberPaths = map (member: workspaceDir + "/${member}") workspaceManifest.workspace.members;

  overrides = {
    sel4-sys = {
      extraPaths = [
        "build"
      ];
    };
    sel4-bitfield-parser = {
      extraPaths = [
        "grammar.pest"
      ];
    };
    sel4-kernel-loader = {
      extraPaths = [
        "asm"
      ];
    };
    tests-root-task-c = {
      extraPaths = [
        "cbits"
      ];
    };
    banscii-assistant-core = {
      resolveLinks = true;
      extraPaths = [
        "assets"
      ];
    };
    mbedtls-sys-auto = {
      extraPaths = [
        "build"
        "vendor"
      ];
    };
    mbedtls = {
      extraPaths = [
        "benches"
        "examples"
        "tests"
      ];
    };
  };

  crates = lib.listToAttrs (lib.forEach workspaceMemberPaths (cratePath: rec {
    name = (crateUtils.crateManifest cratePath).package.name; # TODO redundant
    value = crateUtils.mkCrate cratePath (overrides.${name} or {});
  }));

  augmentedCrates = crateUtils.augmentCrates crates;

in {
  crates = augmentedCrates;
}
