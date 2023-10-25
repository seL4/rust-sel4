#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

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
    rev = "6e1c2fe0637e01b303506cfbad8ed5f863c2446a"; # branch coliasgroup
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
      rev = "1c7a0cb549021bc0781b49aa69359ee8d035981c"; # branch "rust"
      local = localRoot + "/seL4";
    };

    rust-microkit = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "7b8c552b36fe13b8a846b06a659c23697b7df926"; # branch "rust-microkit"
      local = localRoot + "/seL4";
    };

    rust-sel4test = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "b31b459876e01008336394bfa90388d8729ff5f5"; # rust-sel4test
      local = localRoot + "/seL4";
    };
  };

  microkit = fetchGit {
    url = "https://github.com/coliasgroup/microkit.git";
    rev = "80069432ef126411128f2e2c3c95a6d171cb83fe"; # branch "rust-nix"
    local = localRoot + "/microkit";
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
