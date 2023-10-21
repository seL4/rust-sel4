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
      rev = "b170676396064a45e88a8394ae5f463ab792b63a"; # branch "rust"
      local = localRoot + "/seL4";
      # useLocal = true;
    };

    rust-microkit = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "d5196f79d696f74ce40ad13c393a525d1b83ae1f"; # branch "rust-microkit"
      local = localRoot + "/seL4";
      # useLocal = true;
    };

    rust-sel4test = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "06fc7f562f7a78d753687d9dcfda852b834169c5"; # rust-sel4test
      local = localRoot + "/seL4";
      # useLocal = true;
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
    # useLocal = true;
  });

  pythonCapDLTool = fetchGit (capdlCommon // {
    andThen = "/python-capdl-tool";
    # useLocal = true;
  });

  objectSizes = fetchGit (capdlCommon // {
    andThen = "/object_sizes";
    # useLocal = true;
  });
}
