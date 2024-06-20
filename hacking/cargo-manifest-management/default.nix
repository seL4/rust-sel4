#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

rec {
  pkgs = (import ../nix {}).pkgs.build;

  inherit (pkgs) lib;

  makeBlueprint = import ./tool/make-blueprint.nix {
    inherit lib;
  };

  manifestScope = import ./manifest-scope.nix {
    inherit lib;
  };

  manualManifests = import ./manual-manifests.nix {
    inherit lib;
  };

  workspace = makeBlueprint {
    inherit manifestScope manualManifests;
    workspaceRoot = toString ../..;
    workspaceDirFilter = relativePathSegments: lib.head relativePathSegments == "crates";
  };

  inherit (workspace) blueprint debug;

  prettyJSON = (pkgs.formats.json {}).generate;

  blueprintJSON = prettyJSON "blueprint.json" blueprint;
}
