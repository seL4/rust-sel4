{ lib
, callPackage
, runCommand, writeText, writeScript, linkFarm
, runtimeShell
, python3, python3Packages
, crateUtils
}:

rec {

  ensureDot = s: if lib.hasPrefix "." s then s else "./${s}";

  simplePathSegs = path:
    let
      segs = lib.splitString "/" path;
    in
      assert segs != [];
      assert lib.length segs == 1 || lib.all (seg: seg != []) (lib.tail segs);
      assert lib.all (seg: seg != "." && seg != "..") segs;
      segs;

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
      aSegs = simplePathSegs a;
      bSegs = simplePathSegs b;
    in
    assert (lib.head aSegs == "") == (lib.head bSegs == "");
    let
      commonPrefixLen = lib.length (takeWhile lib.id (lib.zipListsWith (x: y: x == y) aSegs bSegs));
      aDeviatingSegs = lib.drop commonPrefixLen aSegs;
      bDeviatingSegs = lib.drop commonPrefixLen bSegs;
      relSegs = lib.genList (lib.const "..") (lib.length aDeviatingSegs) ++ bDeviatingSegs;
    in
      lib.concatStringsSep "/" relSegs;

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
    import difflib
    import json
    import sys
    import toml

    a_path = sys.argv[1]
    b_path = sys.argv[2]

    def read(f):
      return json.dumps(toml.load(f), indent=2, sort_keys=True).splitlines()

    with open(a_path) as f:
      a = read(f)

    with open(b_path) as f:
      b = read(f)

    for line in difflib.unified_diff(a, b, fromfile=a_path, tofile=b_path, lineterm=""):
      print(line)

    if a != b:
      sys.exit(1)
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
