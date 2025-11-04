#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib }:

let
  scanDirForFilesWithName = dirFilter: fileName:
    let
      f = relativePathSegments: absolutePath: lib.concatLists (lib.mapAttrsToList (name: type:
        let
          childAbsolutePath = "${absolutePath}/${name}";
          childRelativePathSegments = relativePathSegments ++ [ name ];
        in {
          "regular" = lib.optional (name == fileName) childAbsolutePath;
          "directory" = lib.optionals (dirFilter childRelativePathSegments) (f childRelativePathSegments childAbsolutePath);
        }."${type}" or []
      ) (builtins.readDir absolutePath));
    in
      f [];

  parseManifestExpr =
    let
      elaborateNix =
        { frontmatter ? null
        , formatPolicyOverrides ? []
        , justEnsureEquivalence ? false
        }:
        {
          inherit frontmatter formatPolicyOverrides justEnsureEquivalence;
        };
    in
      { nix ? {}, ... } @ args:
      let
        elaboratedNix = elaborateNix nix;
        manifestValue = removeAttrs args [ "nix" ];
      in elaboratedNix // {
        inherit manifestValue;
      };

in

{ workspaceRoot, workspaceDirFilter
, manifestScope, manualManifests
}:

let
  cargoNixPaths =
    scanDirForFilesWithName workspaceDirFilter "Cargo.nix" workspaceRoot;

  generateManifest = cargoNixAbsolutePath:
    let
      absolutePath = builtins.dirOf cargoNixAbsolutePath;
      manifestExpr = callManifest {
        inherit absolutePath;
        f = import cargoNixAbsolutePath;
      };
      parsed = parseManifestExpr manifestExpr;
      inherit (parsed) manifestValue;
    in parsed // {
      inherit absolutePath;
      packageName = manifestValue.package.name or null;
      packageVersion = manifestValue.package.version or null;
    };

  callManifest = { absolutePath, f }:
    let
      scope = manifestScope {
        inherit packages absolutePath;
      };
      args = builtins.intersectAttrs (lib.functionArgs f) scope;
    in
      f args;

  generatedManifestsList = map generateManifest cargoNixPaths;

  generatedManifestsByPackageName =
    lib.listToAttrs
      (lib.flip lib.concatMap generatedManifestsList
        (manifest: lib.optional (manifest.packageName != null) (lib.nameValuePair manifest.packageName {
          inherit (manifest) absolutePath manifestValue;
        })));

  manualManifestsByPackageName =
    lib.mapAttrs (_: absolutePath: {
      inherit absolutePath;
      manifestValue = lib.importTOML absolutePath;
    }) manualManifests;

  packages = generatedManifestsByPackageName // manualManifestsByPackageName;

in rec {
  blueprint = generatedManifestsList;

  debug = {
    byPath = lib.listToAttrs (lib.forEach blueprint (attrs: {
      name = attrs.absolutePath;
      value = attrs;
    }));
    byCrateName = lib.listToAttrs
      (lib.forEach
        (lib.filter (attrs: attrs.manifestValue.package.name or null != null) blueprint)
        (attrs: {
          name = attrs.manifestValue.package.name;
          value = attrs;
        }));
  };
}
