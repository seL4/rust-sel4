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
    rev = "f47bacacb6f5cc81934b6ea3116ef95f27793195";
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
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "656f11fd66139e1102c8bae0b07639a2ada9af78"; # branch "rust"
      local = localRoot + "/seL4";
      # useLocal = true;
    };

    rust-sel4cp = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "fc80c9ad05d33e77a6b850dae8eb4b8317ec32a1"; # branch "rust-sel4cp"
    };

    rust-sel4test = fetchGit {
      url = "https://github.com/coliasgroup/seL4.git";
      rev = "e98505aaf5373d1ffc0ee363fbc8182c5795ef2c"; # rust-sel4test
    };
  };

  sel4cp = fetchGit {
    url = "https://github.com/coliasgroup/sel4cp.git";
    rev = "e8d3350fb1f06c5ad3a436be1f09de89d97370e8"; # branch "rust-seL4-nix"
    local = localRoot + "/sel4cp";
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
