#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

rec {
  pkgs = (import ../nix {}).pkgs.build;

  utils = pkgs.callPackage ./utils {};

  manifestScope = pkgs.callPackage ./manifest-scope.nix {};

  manualManifests = import ./manual-manifests.nix;

  workspace = pkgs.callPackage ./workspace.nix {
    inherit utils manifestScope manualManifests;
    workspaceRoot = toString ../..;
  };

  inherit (workspace) script;
}
