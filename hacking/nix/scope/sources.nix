{ lib
}:

let
  fetchGit =
    { url, rev
    , local ? null
    , useLocal ? false
    , andThen ? ""
    }:

    assert useLocal -> local != null;

    let
      remote = builtins.fetchGit rec {
        inherit url rev;
        ref = mkKeepRef rev;
      };
      base = if useLocal then (lib.cleanSource local) else remote;
    in
      base + andThen;

  capdlCommon = {
    url = "https://github.com/coliasgroup/capdl.git";
    rev = "dcad98b8a6665c8ea2b822e97cf9bffbd3c349fe";
    local = localRoot + "/capdl";
  };

  srcRoot = ../../..;

  # TODO
  localRoot = srcRoot + "/../x";

  mkKeepRef = rev: "refs/tags/keep/${builtins.substring 0 32 rev}";

in rec {
  inherit srcRoot localRoot;
  inherit fetchGit mkKeepRef;

  seL4 = {
    rust = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "656f11fd66139e1102c8bae0b07639a2ada9af78"; # branch "rust"
      local = localRoot + "/seL4";
      # useLocal = true;
    };

    rust-sel4cp = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "fc80c9ad05d33e77a6b850dae8eb4b8317ec32a1"; # branch "rust-sel4cp"
      local = localRoot + "/seL4";
      # useLocal = true;
    };

    rust-sel4test = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "07266a57e1073e4dfb4a4f9dddc31b197abdb2a0"; # rust-sel4test
      local = localRoot + "/seL4";
      # useLocal = true;
    };
  };

  sel4cp = fetchGit {
    url = "https://github.com/coliasgroup/sel4cp.git";
    rev = "8c361140e483137d0d95d0aaef1fcdc2d77d07ba"; # branch "rust-seL4-nix"
    local = localRoot + "/sel4cp";
    # useLocal = true;
  };

  capdlTool = fetchGit (capdlCommon // {
    andThen = "/capDL-tool";
  });

  pythonCapDLTool = fetchGit (capdlCommon // {
    andThen = "/python-capdl-tool";
  });

  objectSizes = fetchGit (capdlCommon // {
    andThen = "/object_sizes";
  });
}
