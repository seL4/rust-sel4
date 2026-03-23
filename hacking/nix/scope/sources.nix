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
    rev = "099f46fd10f2fbb34b9fd640f9ba96096dd636d5"; # branch rust-testing
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
    rev = "ebbda2af5aeb5a93d42f108f9732c1b4134d4d64"; # master
    local = localRoot + "/seL4";
  };

  microkit = fetchGit {
    url = "https://github.com/coliasgroup/microkit.git";
    rev = "f68fd8fd5ddcb26ffbafaa81e0b615288a38ee21"; # branch "rust-testing", based on main
    local = localRoot + "/microkit";
    extraFilter = path: type:
      lib.hasSuffix "/target" path;
  };

  sdfgen = fetchGit {
    url = "https://github.com/coliasgroup/microkit_sdf_gen";
    rev = "6a42abbe6a81af3a37cd1070d47106c55c7f4e0c"; # branch "rust", based on main
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
    rev = "1905749cbde7fcda827d95ea8a85924bbcfcff6c"; # lionsos HEAD
    # ref = null;
    local = localRoot + "/lionsos/dep/sddf";
  };

  lionsosAttrs = {
    url = "https://github.com/au-ts/lionsos";
    rev = "1905749cbde7fcda827d95ea8a85924bbcfcff6c"; # HEAD
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
