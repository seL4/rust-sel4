#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib }:

let
  parseSimplePath = path:
    let
      isAbsolute = lib.hasPrefix "/" path;
      pathWithoutLeadingSlash = lib.substring (if isAbsolute then 1 else 0) (lib.stringLength path) path;
      segments = if pathWithoutLeadingSlash == "" then [] else lib.splitString "/" pathWithoutLeadingSlash;
    in
      assert lib.all (seg: seg != "" && seg != "." && seg != "..") segments;
      {
        inherit isAbsolute segments;
      };

  takeWhile = pred:
    let
      f = acc: xs:
        if xs == []
        then acc
        else (
          let
            x = lib.head xs;
            xs' = lib.tail xs;
          in
            if pred x then f (acc ++ [x]) xs' else acc
        );
    in
      f [];

  pathBetween = a: b:
    let
      aParsed = parseSimplePath a;
      bParsed = parseSimplePath b;
    in
    assert aParsed.isAbsolute == bParsed.isAbsolute;
    let
      commonPrefixLen = lib.length (takeWhile lib.id (lib.zipListsWith (x: y: x == y) aParsed.segments bParsed.segments));
      aDeviatingSegs = lib.drop commonPrefixLen aParsed.segments;
      bDeviatingSegs = lib.drop commonPrefixLen bParsed.segments;
      relSegs = lib.genList (lib.const "..") (lib.length aDeviatingSegs) ++ bDeviatingSegs;
    in
      lib.concatStringsSep "/" relSegs;

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
    let
      dirFilter = relativePathSegments: lib.head relativePathSegments == "crates";
    in
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

  generatedManifestsList = map generateManifest cargoNixPaths;

  generatedManifestsByPackageName =
    lib.listToAttrs
      (lib.flip lib.concatMap generatedManifestsList
        (manifest: lib.optional (manifest.packageName != null) (lib.nameValuePair manifest.packageName manifest)));

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
