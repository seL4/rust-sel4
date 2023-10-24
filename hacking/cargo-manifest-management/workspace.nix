#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, callPackage
, writeText, linkFarm, writeScript
, runtimeShell
, python3

, utils, manifestScope, manualManifests
, workspaceRoot
}:

let
  inherit (utils) pathBetween scanDirForFilesWithName renderManifest;

  generatedManifestSources =
    let
      dirFilter = relativePathSegments: lib.head relativePathSegments == "crates";
    in
      scanDirForFilesWithName dirFilter "Cargo.nix" workspaceRoot;

  genrateManifest = cargoNixAbsolutePath:
    let
      absolutePath = builtins.dirOf cargoNixAbsolutePath;
      manifestExpr = callManifest {
        inherit absolutePath;
        f = import cargoNixAbsolutePath;
      };
      parsed = parseManifestExpr manifestExpr;
      inherit (parsed) manifestValue frontmatter justEnsureEquivalence;
    in {
      inherit absolutePath manifestValue frontmatter justEnsureEquivalence;
      packageName = manifestValue.package.name or null;
      packageVersion = manifestValue.package.version or null;
      manifestTOML = renderManifest {
        inherit manifestValue frontmatter;
      };
    };

  parseManifestExpr =
    let
      elaborateNix =
        { frontmatter ? null
        , justEnsureEquivalence ? false
        }:
        {
          inherit
            frontmatter
            justEnsureEquivalence
          ;
        };
    in
      { nix ? {}, ... } @ args:
      let
        elaboratedNix = elaborateNix nix;
        manifestValue = removeAttrs args [ "nix" ];
      in {
        inherit manifestValue;
        inherit (elaboratedNix) frontmatter justEnsureEquivalence;
      };

  callManifest = { absolutePath, f }:
    let
      bespokeScope = manifestScope // {
        localCrates = mkDeps absolutePath;
      };
      args = builtins.intersectAttrs (lib.functionArgs f) bespokeScope;
    in
      f args;

  mkDeps = absolutePath:
    let
      rel = pathBetween absolutePath;
      generated = lib.mapAttrs (_: manifest: { path = rel manifest.absolutePath; }) generatedManifestsByPackageName;
      manual = lib.mapAttrs (_: otherPath: { path = rel otherPath; }) manualManifests;
    in
      generated // manual;

  generatedManifestsList = map genrateManifest generatedManifestSources;

  generatedManifestsByPackageName =
    lib.listToAttrs
      (lib.flip lib.concatMap generatedManifestsList
        (manifest: lib.optional (manifest.packageName != null) (lib.nameValuePair manifest.packageName manifest)));

  plan = lib.listToAttrs (lib.forEach generatedManifestsList (manifest: {
    name = "${manifest.absolutePath}/Cargo.toml";
    value = {
      src = manifest.manifestTOML;
      inherit (manifest) justEnsureEquivalence;
    };
  }));

  planJSON = writeText "plan.json" (builtins.toJSON plan);

  script = writeScript "execute-plan.sh" ''
    #!${runtimeShell}
    exec ${python3.withPackages (p: [ p.toml ])}/bin/python3 ${./execute_plan.py} --plan ${planJSON} "$@"
  '';

  # for manual inspection, useful for debugging this script
  links = linkFarm "crates"
    (lib.mapAttrs'
      (absolutePath: v: lib.nameValuePair "root/${absolutePath}" v.src)
      plan);

in {
  inherit script;

  # for debugging
  inherit generatedManifestsByPackageName links;
}
