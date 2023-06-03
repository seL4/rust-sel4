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
    url = "https://gitlab.com/coliasgroup/capdl.git";
    rev = "63652ffc60e2fbc3f88c275429c1eeee66e3ac43";
    local = localRoot + "/capdl";
  };

  srcRoot = ../../..;

  # TODO
  localRoot = srcRoot + "/../x";

  mkKeepRef = rev: "refs/tags/keep/${builtins.substring 0 32 rev}";

in rec {
  inherit srcRoot localRoot;
  inherit fetchGit mkKeepRef;

  seL4 = {
    rust = fetchGit {
      url = "https://gitlab.com/coliasgroup/seL4.git";
      rev = "862b34791e2e3720bdafc74395469c1b4b97807b"; # branch "rust"
    };

    rust-sel4cp = fetchGit {
      url = "https://gitlab.com/coliasgroup/seL4.git";
      rev = "791d1965fbced4250bdeba41b7454f8e72c19345"; # branch "rust-sel4cp"
      # useLocal = true;
      # local = sources.localRoot + "/seL4";
    };

    rust-sel4test = fetchGit {
      url = "https://gitlab.com/coliasgroup/seL4.git";
      rev = "0b3c3d9672cf742dc948977312216703132f4a29"; # rust-sel4test
    };
  };

  sel4cp = fetchGit {
    url = "https://gitlab.com/coliasgroup/sel4cp.git";
    rev = "e8d3350fb1f06c5ad3a436be1f09de89d97370e8"; # branch "rust-seL4-nix"
    # useLocal = true;
    # local = sources.localRoot + "/sel4cp";
  };

  capdlTool = fetchGit (capdlCommon // {
    andThen = "/capDL-tool";
  });

  pythonCapDLTool = fetchGit (capdlCommon // {
    andThen = "/python-capdl-tool";
    useLocal = true;
  });

  objectSizes = fetchGit (capdlCommon // {
    andThen = "/object_sizes";
  });
}
