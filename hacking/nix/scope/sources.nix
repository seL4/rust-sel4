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
      rev = "2404473ed2e68c8e627cb4d6588bd57633038109"; # branch "rust"
      local = localRoot + "/seL4";
    };

    rust-microkit = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "5f71dc4a5cd90b823876bec10265da92eae486ab"; # branch "rust-microkit"
      local = localRoot + "/seL4";
    };

    rust-sel4test = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "e1b3a2bfc5f9d122f3cebc5b6f2751b83f0e97ac"; # rust-sel4test
      local = localRoot + "/seL4";
    };
  };

  microkit = fetchGit {
    url = "https://github.com/coliasgroup/microkit.git";
    rev = "5f7b2da0a6233b4cc763d3dff4451532958a8877"; # branch "rust-nix"
    local = localRoot + "/microkit";
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
