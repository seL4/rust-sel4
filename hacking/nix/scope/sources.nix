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
    , extraFilter ? _: _: true
    }:

    assert useLocal -> local != null;

    let
      remote = builtins.fetchGit {
        inherit url rev ref submodules;
      };
      cleanedSource = lib.cleanSourceWith {
        src = local;
        filter = path: type: lib.cleanSourceFilter path type || extraFilter path type;
      };
      base = if useLocal then cleanedSource else remote;
    in
      base + andThen;

  capdlCommon = {
    url = "https://github.com/coliasgroup/capdl.git";
    rev = "6a8c249a2afc78c285865c75c3b7cb2664014269"; # branch rust-testing
    local = localRoot + "/capdl";
    extraFilter = path: type:
      lib.hasSuffix "/.stack-work" path || lib.hasSuffix "/stack.yaml.lock" path;
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

  seL4 = fetchGit {
    url = "https://github.com/seL4/seL4.git";
    rev = "fdf7f2a0823330615dfb6b3ae6706403217e74ea";
    local = localRoot + "/seL4";
  };

  microkit = fetchGit {
    url = "https://github.com/coliasgroup/microkit.git";
    rev = "26230271a12cb39c6cc08ac75d33e02489990922"; # branch "rust-nix", based on 3.0.0
    local = localRoot + "/microkit";
    extraFilter = path: type:
      lib.hasSuffix "/target" path;
  };

  sdfgen = fetchGit {
    url = "https://github.com/coliasgroup/microkit_sdf_gen";
    rev = "16f735ab4954ae8e0490c65787b33f5b0ec87957"; # branch "rust", based on 0.27.0
    local = localRoot + "/microkit_sdf_gen";
    extraFilter = path: type:
      lib.hasSuffix "/.zig-cache" path || lib.hasSuffix "/zig-out" path || lib.hasSuffix "/result" path;
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
