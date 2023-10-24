#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, callPackage
, runCommand, writeText, writeScript, linkFarm
, runtimeShell
, python3, python3Packages
}:

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

  renderManifest = { manifestValue, frontmatter ? null }:
    let
      frontmatterCatArg = lib.optionalString
        (frontmatter != null)
        (builtins.toFile "frontmatter.txt" "${frontmatter}\n");

      # TODO figure out what's going on here
      rustfmtLdHack = "LD_LIBRARY_PATH=${rustToolchainForRustfmtWithTOMLSupport}/lib";
    in
      runCommand "Cargo.toml" {
        nativeBuildInputs = [
          python3Packages.toml
          rustfmtWithTOMLSupport
        ];
        manifestJSON = builtins.toJSON manifestValue;
        passAsFile = [ "manifestJSON" ];
      } ''
        python3 ${./coarsely_stylize_manifest.py} < $manifestJSONPath > Cargo.toml
        ${rustfmtLdHack} rustfmt --config format_cargo_toml=true Cargo.toml
        cat ${frontmatterCatArg} Cargo.toml >> $out
      '';

  fenix =
    let
      rev = "0fbf5652c9596038c0f4784d3e93d1f1a543e24b";
      src = fetchTarball "https://github.com/nix-community/fenix/archive/${rev}.tar.gz";
    in import src {};

  rustToolchainForRustfmtWithTOMLSupport = (fenix.toolchainOf {
    channel = "nightly";
    date = "2023-07-01";
    sha256 = "sha256-pWd4tAHP4QWGC3CKWZzDjzYANxATC6CGRmKuP2ZZv5k=";
  }).completeToolchain;

  rustfmtWithTOMLSupport = callPackage ./rustfmt-with-toml-support.nix {
    rustToolchain = rustToolchainForRustfmtWithTOMLSupport;
  };

in {
  inherit pathBetween scanDirForFilesWithName renderManifest;
}
