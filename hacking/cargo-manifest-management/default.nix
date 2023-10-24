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

  inherit (workspace) plan;

  prettyJSON = name: value:
    with pkgs;
    runCommand name {
      nativeBuildInputs = [
        jq
      ];
    } ''
      jq < ${builtins.toFile "plan.json" (builtins.toJSON value)} > $out
    '';

  planJSON = prettyJSON "plan.json" plan;

  # for debugging:

  test = prettyJSON "Crate.json" testValue;

  testValue = getManifestByPackageName "sel4";

  getManifestByPackageName = name: lib.head
    (lib.filter
      (v: v.package.name or null == name)
      (map
        (v: v.manifest)
        (lib.attrValues plan)));
}
