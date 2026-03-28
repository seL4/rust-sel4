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
    rev = "6282fc3d4cf4a2d17c52dcdea11b08997458f9a3"; # branch rust-testing
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
    rev = "27a52ddd4cf592b1996ba50074c85e21d832398a"; # master
    local = localRoot + "/seL4";
  };

  microkit = fetchGit {
    url = "https://github.com/seL4/microkit.git";
    rev = "fc90d89013b3a0cb330d2ac8d61abaee1223f711"; # main
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
    rev = "b0e86b1d5082ec8680585ad3429a8a519efa57ff"; # lionsos HEAD
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
