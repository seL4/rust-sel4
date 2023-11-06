#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

rec {
  pkgs = (import ../nix {}).pkgs.build;

  inherit (pkgs) lib;

  tool = import ./tool.nix {
    inherit lib;
  };

  manifestScope = import ./manifest-scope.nix {
    inherit lib;
  };

  manualManifests = import ./manual-manifests.nix;

  workspace = tool {
    inherit manifestScope manualManifests;
    workspaceRoot = toString ../..;
    workspaceDirFilter = relativePathSegments: lib.head relativePathSegments == "crates";
  };

  inherit (workspace) generatedManifestsList;

  x = pkgs.callPackage ./x.nix {
    inherit generatedManifestsList;
  };

  inherit (x) witness updateScript debug;
}
