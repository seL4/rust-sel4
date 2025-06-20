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
    , ref ? mkKeepRef rev
    , submodules ? false
    , local ? null
    , useLocal ? false
    , andThen ? ""
    }:

    assert useLocal -> local != null;

    let
      remote = builtins.fetchGit {
        inherit url rev ref submodules;
      };
      base = if useLocal then (lib.cleanSource local) else remote;
    in
      base + andThen;

  capdlCommon = {
    url = "https://github.com/coliasgroup/capdl.git";
    rev = "6ea203e8ebbcf8d786939aa706262b5073aae676"; # branch rust-testing
    local = localRoot + "/capdl";
  };

  srcRoot = ../../..;

  # TODO
  # localRoot = srcRoot + "/tmp/src";
  localRoot = srcRoot + "/../x";

  mkKeepRef = rev: "refs/tags/keep/${builtins.substring 0 32 rev}";

in rec {
  inherit srcRoot localRoot;
  inherit fetchGit mkKeepRef;

  # NOTE: Be sure to keep the commit hashes in the top-level README up-to-date.

  seL4 = {
    rust = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "dcdd2bcf2a64f3b34d2a406824fa54df7ed55571"; # branch "rust-testing", based on 13.0.0
      local = localRoot + "/seL4";
    };

    rust-microkit = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "4b97df4c7e24fd0c297e21cae8d997a08b8952b0"; # branch "rust-microkit" seL4/seL4:microkit
      local = localRoot + "/seL4";
    };
  };

  microkit = fetchGit {
    url = "https://github.com/coliasgroup/microkit.git";
    rev = "58ef533a399b916ab39df0c93e2741993614ba4f"; # branch "rust-nix", based on 2.0.1
    local = localRoot + "/microkit";
  };

  sdfgen = fetchGit {
    url = "https://github.com/au-ts/microkit_sdf_gen";
    rev = "232ad1a5425899b0fb017dfd19ff626b0223f812";
    ref = "0.23.1";
    local = localRoot + "/microkit_sdf_gen";
  };

  # sddf = fetchGit (lionsosAttrs // {
  #   andThen = "/dep/sddf";
  # });

  sddf = fetchGit {
    url = "https://github.com/au-ts/sddf";
    rev = "50c4eb17a7dfc65b4111e4770227c3919bdaa1c3";
    ref = "0.6.0";
    local = localRoot + "/lionsos/dep/sddf";
    # useLocal = true;
  };

  lionsosAttrs = {
    url = "https://github.com/au-ts/lionsos";
    rev = "681143753fc8c7b91153a8fe1486674a70fbb0eb";
    ref = "0.3.0";
    local = localRoot + "/lionsos";
    # useLocal = true;
  };

  lionsos = fetchGit lionsosAttrs;

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
