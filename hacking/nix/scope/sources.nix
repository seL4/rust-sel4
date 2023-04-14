{ lib
}:

let
  fetchGitOrLocalAndThen =
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
    url = "https://gitlab.com/coliasgroup/capdl.git";
    rev = "af3c4b937982bb003f5a7fecd2958e25f1cd84de";
    local = localRoot + "/capdl";
  };

  srcRoot = ../../..;

  # TODO
  localRoot = srcRoot + "/../../../../x";

  mkKeepRef = rev: "refs/tags/keep/${builtins.substring 0 32 rev}";

in rec {
  inherit srcRoot localRoot;
  inherit mkKeepRef;

  capdlTool = fetchGitOrLocalAndThen (capdlCommon // {
    andThen = "/capDL-tool";
  });

  pythonCapDLTool = fetchGitOrLocalAndThen (capdlCommon // {
    andThen = "/python-capdl-tool";
    useLocal = true;
  });

  objectSizes = fetchGitOrLocalAndThen (capdlCommon // {
    andThen = "/object_sizes";
  });
}
