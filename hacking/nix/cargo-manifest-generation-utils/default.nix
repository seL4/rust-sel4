{ lib, stdenv, buildPackages, buildPlatform, hostPlatform
, writeText, linkFarm, emptyFile, runCommand, writeScript
, runtimeShell
, python3, python3Packages
, callPackage
, crateUtils
}:

rec {

  ensureDot = s: if lib.hasPrefix "." s then s else "./${s}";

  pathBetween = here: there: import (runCommand "path-between.nix" {
    nativeBuildInputs = [ python3 ];
  } ''
    python3 -c 'from os.path import relpath; print("\"{}\"".format(relpath("${there}", "${here}")))' > $out
  '');

  mkCrate =
    let
      elaborateNix =
        { path
        , local ? {}
        , frontmatter ? null
        , justEnsureEquivalence ? false
        , meta ? {}
        , passthru ? {}
        }:
        {
          inherit
            path
            local
            frontmatter
            justEnsureEquivalence
            passthru
          ;

          meta = elaborateMeta meta;
        };

      elaborateMeta =
        { labels ? {}
        , requirements ? {}
        , skip ? false
        }:
        {
          inherit
            labels
            requirements
            skip
          ;
        };

    in
      { nix ? {}, ... } @ args:

      let
        elaboratedNix = elaborateNix nix;

        paths = lib.flip lib.mapAttrsRecursive elaboratedNix.local (_: v:
          if !lib.isList v then v else lib.listToAttrs (map (otherCrate: lib.nameValuePair otherCrate.name {
            path = ensureDot (pathBetween
              (toString elaboratedNix.path)
              (toString otherCrate.path));
          }) v)
        );

        manifest = crateUtils.clobber [
          paths
          (removeAttrs args [ "nix" ])
        ];

      in {
        inherit manifest;
        manifestTOML = renderManifest {
          inherit manifest;
          inherit (elaboratedNix) frontmatter;
        };
        inherit (elaboratedNix) path meta justEnsureEquivalence passthru;
      };

  prependFrontmatter = frontmatter: manifest: runCommand manifest.name {} ''
    cat ${builtins.toFile "x" frontmatter} > $out
    echo >> $out
    cat ${manifest} >> $out
  '';

  fixupRustfmt = manifest: runCommand manifest.name {} ''
    sed 's|)" \.|)".|g' < ${manifest} > $out
  '';

  nixToInlineTOML = expr: runCommand "x.toml" {
    nativeBuildInputs = [
      python3Packages.toml
    ];
    json = builtins.toJSON expr;
    passAsFile = [ "json" ];
  } ''
    python3 ${./nix-to-inline-toml.py} < $jsonPath > $out
  '';

  maybePrependFrontmatter = frontmatter:
    if frontmatter != null then prependFrontmatter frontmatter else lib.id
  ;

  renderManifest = crate: maybePrependFrontmatter crate.frontmatter (fixupRustfmt (formatCargoTOML (nixToInlineTOML crate.manifest)));

  # # #

  mkScript =
    { root
    , plan
    , actuallyDoIt
    }:
    let
      mkAction = actuallyDoIt: dst: { src, justCheckEquivalenceWith }: ''
        ${lib.optionalString (!actuallyDoIt) ''
          if ! test -f ${dst}; then
            echo "${dst} does not exist"
            false
          fi
        ''}
        ${
          if justCheckEquivalenceWith != null
          then ''
            ${justCheckEquivalenceWith src dst}
          ''
          else ''
            if ! cmp -s -- ${dst} ${src}; then
              ${
                if actuallyDoIt
                then ''
                  cp -vL --no-preserve=all ${src} ${dst}
                ''
                else ''
                  echo "${dst} differs from ${src}"
                  false
                ''
              }
            fi
          ''
        }
      '';
    in writeScript "x.sh" ''
      #!${runtimeShell}
      set -e
      cd ${root}
      ${lib.concatStrings (lib.mapAttrsToList (mkAction actuallyDoIt) plan)}
    '';

  checkTOMLEquivalence = src: dst: ''
    if ! ${python3.withPackages (p: [ p.toml ])}/bin/python3 ${tomlDiff} ${dst} ${src}; then
      echo "${dst} differs from ${src}"
      false
    fi
  '';

  tomlDiff = writeText "toml-diff.py" ''
    import sys
    import toml

    a = sys.argv[1]
    b = sys.argv[2]

    with open(a) as f:
      a = toml.load(f)

    with open(b) as f:
      b = toml.load(f)

    assert(a == b)
  '';

  # # #

  fenixRev = "a9a262cbec1f1c3f771071fd1a246ee05811f5a1";
  fenixSource = fetchTarball "https://github.com/nix-community/fenix/archive/${fenixRev}.tar.gz";
  fenix = import fenixSource {};

  formatCargoTOML = callPackage ./format-cargo-toml.nix {
    inherit rustfmtWithTOMLSupport;
  };

  rustToolchainForRustfmtWithTOMLSupport = fenix.toolchainOf {
    channel = "nightly";
    date = "2022-08-06";
    sha256 = "sha256-LcLjNIjif2Li6eJEYEe28E9W/+09lgQcgnkGyqtGlns=";
  };

  rustfmtWithTOMLSupport = callPackage ./rustfmt-with-toml-support.nix {
    rustToolchain = rustToolchainForRustfmtWithTOMLSupport.completeToolchain;
  };

}
