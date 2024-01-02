#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPackages
, linkFarm, emptyDirectory
, crateUtils
, vendorLockfile, pruneLockfile
, defaultRustToolchain, defaultRustTargetInfo
} @ outerArgs:

{ superLockfile
}:

let

  superVendoredLockfile = vendorLockfile { lockfile = superLockfile; };

  runClippyDefault =
    # true
    false
  ;

in

{ rootCrate

, layers ? [ crateUtils.defaultIntermediateLayer ] # default to two building in two steps (external then local)
, commonModifications ? {}
, lastLayerModifications ? {}

, test ? false

, release ? false
, profile ? if release then "release" else null

, features ? []
, noDefaultFeatures ? false

, rustToolchain ? defaultRustToolchain
, rustTargetInfo ? defaultRustTargetInfo

, stdenv ? outerArgs.stdenv

, runClippy ? runClippyDefault
}:

let
  inherit (crateUtils) composeModifications elaborateModifications;

  accumulate =
    let
      f = acc: prev: xs:
        if lib.length xs == 0
        then acc
        else
          let
            cur = prev // lib.head xs;
            acc' = acc ++ [ cur ];
          in
            f acc' cur (lib.tail xs);
    in
      f [] {};

  lockfile = builtins.toFile "Cargo.lock" lockfileContents;
  lockfileContents = builtins.readFile lockfileDrv;
  lockfileDrv = pruneLockfile {
    superLockfile = superLockfile;
    superLockfileVendoringConfig = superVendoredLockfile.configFragment;
    rootCrates = [ rootCrate ];
    extraManifest = elaboratedCommonModifications.modifyManifest {}; # TODO
  };

  elaboratedCommonModifications = elaborateModifications commonModifications;

  closure = crateUtils.getClosureOfCrate rootCrate;

  accumulatedLayers =
    let
      expandLayer = layer: crateUtils.getClosureOfCrates (map (key: closure.${key}) layer);
      accumulatedCratesByLayer = accumulate (map expandLayer (map (lib.getAttr "crates") layers));
    in
      lib.zipListsWith
        (layer: reals: {
          inherit reals;
          inherit (layer) modifications;
        })
        layers
        accumulatedCratesByLayer
      ;

  lastIntermediateLayer = f (lib.reverseList accumulatedLayers);

  baseArgs = {
    depsBuildBuild = [ buildPackages.stdenv.cc ];
    nativeBuildInputs = [ rustToolchain ];
  } // lib.optionalAttrs (rustTargetInfo.path != null) {
    RUST_TARGET_PATH = rustTargetInfo.path;
  };

  baseManifest = {
    workspace.resolver = "2";
    workspace.members = [ "src/${rootCrate.name}" ];
  };

  baseConfig = crateUtils.clobber [
    (crateUtils.baseConfig {
      inherit rustToolchain;
      rustTargetName = rustTargetInfo.name;
    })
    {
      unstable.unstable-options = true;
    }
    (vendorLockfile { inherit lockfileContents; }).configFragment
  ];

  baseFlags = [
    "--offline"
    "--frozen"
    "-p" rootCrate.name
  ] ++ lib.optionals (lib.length features > 0) [
    "--features" (lib.concatStringsSep "," features)
  ] ++ lib.optionals noDefaultFeatures [
    "--no-default-features"
  ] ++ lib.optionals (profile != null) [
    "--profile" profile
  ] ++ [
    "--target" rustTargetInfo.name
    "-j" "$NIX_BUILD_CORES"
  ];

  cargoSubcommand = if test then "test" else "build";

  mkCargoInvocation = commonArgs: subcommandArgs:
    let
      joinedCommonArgs = lib.concatStringsSep " " commonArgs;
      joinedSubcommandArgs = lib.concatStringsSep " "
        (subcommandArgs ++ lib.optionals test [
          "--no-run"
        ]);
    in ''
      ${lib.optionalString runClippy ''
        cargo clippy ${joinedCommonArgs} -- -D warnings
      ''}
      cargo ${cargoSubcommand} ${joinedCommonArgs} ${joinedSubcommandArgs}
    '';

  findTestsCommandPrefix = targetDir: [
    "find"
      "${targetDir}/${rustTargetInfo.name}/*/deps"
      "-maxdepth" "1"
      "-executable"
      "-name" "'*.elf'"
  ];

  f = accumulatedLayers:
    if lib.length accumulatedLayers == 0
    then emptyDirectory
    else
      let
        layer = lib.head accumulatedLayers;
        prev = f (lib.tail accumulatedLayers);

        dummies = lib.attrValues (builtins.removeAttrs closure (lib.attrNames layer.reals));
        src = crateUtils.collectRealsAndDummies (lib.attrValues layer.reals) dummies;

        modifications = composeModifications (elaborateModifications layer.modifications) elaboratedCommonModifications;

        manifest = crateUtils.toTOMLFile "Cargo.toml" (modifications.modifyManifest baseManifest);
        config = crateUtils.toTOMLFile "config" (modifications.modifyConfig baseConfig);
        flags = baseFlags ++ modifications.extraCargoFlags;

        workspace = linkFarm "workspace-with-dummies" [
          { name = "Cargo.toml"; path = manifest; }
          { name = "Cargo.lock"; path = lockfile; }
          { name = "src"; path = src; }
        ];
      in
        modifications.modifyDerivation (stdenv.mkDerivation (baseArgs // {
          name = "${rootCrate.name}-intermediate";

          phases = [ "buildPhase" ];

          buildPhase = ''
            runHook preBuild

            cp -r --preserve=timestamps ${prev} $out
            chmod -R +w $out

            ${mkCargoInvocation (flags ++ [
              "--config" "${config}"
              "--manifest-path" "${workspace}/Cargo.toml"
              "--target-dir" "$out"
            ]) []}

            ${lib.optionalString test (lib.concatStringsSep " " (findTestsCommandPrefix "$out" ++ [
              "-delete"
            ]))}

            runHook postBuild
          '';

          passthru = {
            # for debugging
            inherit prev workspace config;
          };
        }));

in let
  src = crateUtils.collectReals (lib.attrValues closure);

  modifications = composeModifications (elaborateModifications lastLayerModifications) elaboratedCommonModifications;

  manifest = crateUtils.toTOMLFile "Cargo.toml" (modifications.modifyManifest baseManifest);
  config = crateUtils.toTOMLFile "config" (modifications.modifyConfig baseConfig);
  flags = baseFlags ++ modifications.extraCargoFlags;

  workspace = linkFarm "workspace" [
    { name = "Cargo.toml"; path = manifest; }
    { name = "Cargo.lock"; path = lockfile; }
    { name = "src"; path = src; }
  ];

  final = modifications.modifyDerivation (stdenv.mkDerivation (baseArgs // {
    name = rootCrate.name;

    phases = [ "buildPhase" ];

    buildPhase = ''
      runHook preBuild

      target_dir=$(realpath ./target)
      cp -r --preserve=timestamps ${lastIntermediateLayer} $target_dir
      chmod -R +w $target_dir

      ${mkCargoInvocation (flags ++ [
        "--config" "${config}"
        "--manifest-path" "${workspace}/Cargo.toml"
        "--target-dir" "$target_dir"
      ]) (lib.optionals (!test) [
        "--out-dir" "$out/bin"
      ])}

      ${lib.optionalString test (lib.concatStringsSep " " (findTestsCommandPrefix "$target_dir" ++ [
        "-exec" "install" "-D" "-t" "$out/bin" "'{}'" "';'"
      ]))}

      runHook postBuild
    '';

    passthru = {
      inherit rootCrate workspace lastIntermediateLayer;
    };
  }));

in final

# NOTE "-Z avoid-dev-deps" for deps of std
