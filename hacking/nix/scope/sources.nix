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
    rev = "21db39ae9ec15a432082b8071797a96951b1daec";
    local = localRoot + "/capdl";
  };

  # TODO
  localRoot =../../../../../../../../x;

in rec {
  inherit mkKeepRef;

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
