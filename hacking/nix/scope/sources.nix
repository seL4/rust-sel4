{ lib
, mkKeepRef
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
    rev = "0ba5e62be342eba112f1868d41d5e2ab723f14ca";
    local = localRoot + "/capdl";
  };

  # TODO
  localRoot =../../../../../../../x;

in rec {
  inherit mkKeepRef;
  inherit localRoot;

  capdlTool = fetchGitOrLocalAndThen (capdlCommon // {
    andThen = "/capDL-tool";
  });

  pythonCapDLTool = fetchGitOrLocalAndThen (capdlCommon // {
    andThen = "/python-capdl-tool";
  });

  objectSizes = fetchGitOrLocalAndThen (capdlCommon // {
    andThen = "/object_sizes";
  });
}
