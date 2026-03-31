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
    rev = "e0dfb895ec7dda40a6e59eec11fe2c20dd5c831d"; # branch rust-testing
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
    rev = "881de507fe528490dc5e570c7810a149bad5880f"; # 15.0.0
    local = localRoot + "/seL4";
  };

  microkit = fetchGit {
    url = "https://github.com/seL4/microkit.git";
    rev = "27e7b6784664a5e62c11b49559995d266dece3c3"; # 2.2.0
    local = localRoot + "/microkit";
    extraFilter = path: type:
      lib.hasSuffix "/target" path;
  };

  sdfgen = fetchGit {
    url = "https://github.com/au-ts/microkit_sdf_gen";
    rev = "e92b69d5f5ceb6819301514fd7704618360ef9dd"; # main
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
    rev = "d1f5252ea64edab6087552eb4220e23c019c2fbe"; # from lionsos main
    # ref = null;
    local = localRoot + "/lionsos/dep/sddf";
  };

  lionsosAttrs = {
    url = "https://github.com/au-ts/lionsos";
    rev = "927fc257c786df3b0b3c2d19673c186e0a166df3"; # main
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
