{ lib, formats
, runCommand, writeText, writeShellApplication
, generatedManifestsList
}:

let
  tomlNormalize = ./target/debug/toml-normalize;

  generateTOML = (formats.toml {}).generate;

  appendNewline = s: "${s}\n";

  normalize = { frontmatter, inputManifestTOML }:
    let
      frontmatterArg = lib.optionalString (frontmatter != null) (
        builtins.toFile "frontmatter.txt" (appendNewline frontmatter)
      );
    in
      runCommand "Cargo.toml" {} ''
        ${tomlNormalize} \
          ${inputManifestTOML} \
          --builtin-policy cargo-manifest \
          | cat ${frontmatterArg} - > $out
      '';

  format = { frontmatter, manifestValue }: normalize {
    inherit frontmatter;
    inputManifestTOML = generateTOML "Cargo.toml" manifestValue;
  };

  augmentedGeneratedManifestsList = lib.forEach generatedManifestsList (attrs: attrs // rec {

    origManifestPath = "${attrs.absolutePath}/Cargo.toml";

    origManifest = builtins.path {
      path = origManifestPath;
    };

    origManifestExists = builtins.pathExists origManifestPath;

    normalizedOrigManifest = normalize {
      inherit (attrs) frontmatter;
      inputManifestTOML = origManifest;
    };

    normalizedManifest = format {
      inherit (attrs) frontmatter manifestValue;
    };

    witness = if attrs.justEnsureEquivalence then witnessEquivalent else witnessIdentical;

    witnessEquivalent = runCommand "witness-structurally-equivalent" {} ''
      if ! diff -u --label ${origManifestPath} ${normalizedOrigManifest} ${normalizedManifest}; then
        echo "${origManifestPath} is not structurally equivalent to ${normalizedManifest}" >&2
        exit 1
      fi
      touch $out
    '';

    witnessIdentical = runCommand "witness-identical" {} ''
      if ! diff -u --label ${origManifestPath} ${origManifest} ${normalizedManifest}; then
        echo "${origManifestPath} is not identical to ${normalizedManifest}" >&2
        exit 1
      fi
      touch $out
    '';
  });

  witness = writeText "witness" (lib.flip lib.concatMapStrings augmentedGeneratedManifestsList (attrs: with attrs;
    appendNewline (if justEnsureEquivalence then witnessEquivalent else witnessIdentical)
  ));

  updateScript = writeShellApplication {
    name = "update";
    text = lib.flip lib.concatMapStrings augmentedGeneratedManifestsList (attrs: with attrs;
      appendNewline (
          if justEnsureEquivalence
          then "# ${witnessEquivalent}"
          else if origManifestExists
          then "# ${witnessIdentical}"
          else "cp ${normalizedManifest} ${origManifestPath}"
        )
    );
  };

in rec {
  inherit witness updateScript;

  # for debugging

  debug = {
    byPath = lib.listToAttrs (lib.forEach augmentedGeneratedManifestsList (attrs: {
      name = attrs.absolutePath;
      value = attrs;
    }));
    byCrateName = lib.listToAttrs
      (lib.forEach
        (lib.filter (attrs: attrs.manifestValue.package.name or null != null) augmentedGeneratedManifestsList)
        (attrs: {
          name = attrs.manifestValue.package.name;
          value = attrs;
        }));
  };
}
