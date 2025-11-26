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
    rev = "c0fc32450fb5e8460083b89a84d067249b109cfc"; # 14.0.0
    local = localRoot + "/seL4";
  };

  microkit = fetchGit {
    url = "https://github.com/coliasgroup/microkit.git";
    rev = "5825477583cad0dd555569373fef797aa82d7047"; # branch "rust-testing", based on 2.1.0
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

  # micropython submodule is too big for this to be worth it
  # sddf = fetchGit (lionsosAttrs // {
  #   submodules = true;
  #   andThen = "/dep/sddf";
  #   local = localRoot + "/lionsos/dep/sddf";
  # });

  sddf = fetchGit {
    url = "https://github.com/au-ts/sddf";
    rev = "6bba35a9597639929be9e0db4b7f8fba2a307ad6"; # lionsos HEAD
    # ref = null;
    local = localRoot + "/lionsos/dep/sddf";
  };

  lionsosAttrs = {
    url = "https://github.com/au-ts/lionsos";
    rev = "67377b5b3b340d1466abf4fb3ca098a5cae36d82"; # HEAD
    # ref = null;
    local = localRoot + "/lionsos";
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
