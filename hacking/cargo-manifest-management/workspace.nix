#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, callPackage, newScope
, writeText, runCommand, linkFarm, writeScript
, runtimeShell
, python3

, utils, manifestScope, manualManifests
, workspaceRoot
}:

let
  inherit (utils) pathBetween scanDirForFilesWithName renderManifest;

  generatedManifestSources =
    let
      dirFilter = relativePathSegments:
        lib.elemAt relativePathSegments 0 == "crates";
    in
      scanDirForFilesWithName dirFilter "Cargo.nix" workspaceRoot;

  generatedManifestsList = lib.forEach generatedManifestSources (absolutePath:
    let
      relativePath =
        pathBetween
          workspaceRoot
          (builtins.dirOf absolutePath);
      manifestExpr = callManifest {
        inherit relativePath;
        f = import absolutePath;
      };
      parsed = parseManifestExpr manifestExpr;
      inherit (parsed) manifestValue frontmatter justEnsureEquivalence;
    in {
      inherit relativePath;
      packageName = manifestValue.package.name or null;
      manifestTOML = renderManifest {
        inherit manifestValue frontmatter;
      };
      inherit manifestValue justEnsureEquivalence;
    });

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

  callManifest = { relativePath, f }:
    let
      bespokeScope = manifestScope // {
        localCrates = mkDeps relativePath;
      };
      args = builtins.intersectAttrs (lib.functionArgs f) bespokeScope;
    in
      f args;

  mkDeps = relativePath:
    let
      rel = pathBetween relativePath;
      generated = lib.mapAttrs (_: manifest: { path = rel manifest.relativePath; }) generatedManifestsByPackageName;
      manual = lib.mapAttrs (_: otherPath: { path = rel otherPath; }) manualManifests;
    in
      generated // manual;

  generatedManifestsByPackageName =
    lib.listToAttrs
      (lib.flip lib.concatMap generatedManifestsList
        (manifest: lib.optional (manifest.packageName != null) (lib.nameValuePair manifest.packageName manifest)));

  plan = lib.listToAttrs (lib.forEach generatedManifestsList (manifest: {
    name = "${manifest.relativePath}${lib.optionalString (manifest.relativePath != "") "/"}Cargo.toml";
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
  links = linkFarm "crates" (lib.mapAttrs (_: v: v.src) plan);

in {
  inherit script;

  # for debugging
  inherit generatedManifestsByPackageName links;
}
