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

  inherit (workspace) blueprint debug;

  prettyJSON = name: value: with pkgs; runCommand name {
    nativeBuildInputs = [ jq ];
  } ''
    jq < ${builtins.toFile name (builtins.toJSON value)} > $out
  '';

  blueprintJSON = prettyJSON "blueprint.json" blueprint;
}
