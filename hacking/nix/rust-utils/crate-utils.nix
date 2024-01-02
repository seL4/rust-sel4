#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, buildPlatform, hostPlatform
, writeText, linkFarm, runCommand
, toTOMLFile
, chooseLinkerForRustTarget ? throw "chooseLinkerForRustTarget must be set"
}:

let
  compose = f: g: x: f (g x);
in

rec {

  inherit toTOMLFile;

  clobber = lib.fold lib.recursiveUpdate {};

  toUpperWithUnderscores = s: lib.replaceStrings ["-"] ["_"] (lib.toUpper s);

  ###

  dummyLibInSrc = dummyInSrc "lib.rs" dummyLib;
  dummyMainWithStdInSrc = dummyInSrc "main.rs" dummyMainWithStd;
  dummyMainWithOrWithoutStdInSrc = dummyInSrc "main.rs" dummyMainWithOrWithoutStd;

  dummyInSrc = name: path:
    let
      src = linkFarm "dummy-src" [
        {
          inherit name path;
        }
      ];
    in
      "${src}/${name}";

  dummyMainWithStd = writeText "main.rs" ''
    fn main() {}
  '';

  # dummyLib = writeText "lib.rs" ''
  #   #![no_std]
  # '';

  # HACK for cargo test (required by harness = "false")
  dummyLib = dummyMainOrLibWithOrWithoutStd;

  dummyMainWithOrWithoutStd = dummyMainOrLibWithOrWithoutStd;

  dummyMainOrLibWithOrWithoutStd = writeText "main_or_lib.rs" ''
    #![cfg_attr(target_os = "none", no_std)]
    #![cfg_attr(target_os = "none", no_main)]
    #![cfg_attr(target_os = "none", feature(lang_items))]
    #![cfg_attr(target_os = "none", feature(core_intrinsics))]

    #[cfg(target_os = "none")]
    #[panic_handler]
    extern fn panic_handler(_: &core::panic::PanicInfo) -> ! {
      core::intrinsics::abort()
    }

    #[cfg(target_os = "none")]
    #[lang = "eh_personality"]
    extern fn eh_personality() {
    }

    #[cfg(not(target_os = "none"))]
    #[allow(dead_code)]
    fn main() {
    }
  '';

  ###

  getClosureOfCrate = root: root.closure;
  getClosureOfCrates = lib.foldl' (acc: crate: acc // getClosureOfCrate crate) {};

  collectReals = reals: collectRealsAndDummies reals [];

  collectDummies = dummies: collectRealsAndDummies [] dummies;

  collectRealsAndDummies = reals: dummies: linkFarm "crates" (map (crate: {
    name = crate.name;
    path = crate.real;
  }) reals ++ map (crate: {
    name = crate.name;
    path = crate.dummy;
  }) dummies);

  ###

  # TODO improve this mechanism
  linkerConfig = { rustToolchain, rustTargetName }@args:
    let
      f = { rustTargetName, platform }:
        let
          linker = chooseLinkerForRustTarget {
            inherit rustToolchain rustTargetName platform;
          };
        in
          lib.optionalAttrs (linker != null) {
            target = {
              "${rustTargetName}".linker = linker;
            };
          };
    in
      clobber [
        (f { rustTargetName = buildPlatform.config; platform = buildPlatform; })
        (f { inherit rustTargetName; platform = hostPlatform; })
      ];

  baseConfig = { rustToolchain, rustTargetName }@args: clobber [
    {
      build.incremental = false;
    }
    (linkerConfig args)
    # {
    #   target."cfg(all())" = {
    #     rustflags = [ "-D" "warnings" ];
    #   };
    # }
  ];

  ###

  defaultIntermediateLayer = {
    crates = [];
    modifications = {};
  };

  elaborateModifications =
    { modifyManifest ? lib.id
    , modifyConfig ? lib.id
    , modifyDerivation ? lib.id
    , extraCargoFlags ? []
    }:
    {
      inherit
        modifyManifest
        modifyConfig
        modifyDerivation
        extraCargoFlags
      ;
    };

  composeModifications = f: g: {
    modifyManifest = compose f.modifyManifest g.modifyManifest;
    modifyConfig = compose f.modifyConfig g.modifyConfig;
    modifyDerivation = compose f.modifyDerivation g.modifyDerivation;
    extraCargoFlags = f.extraCargoFlags ++ g.extraCargoFlags;
  };

  ###

  traverseAttrs = f: attrs: state0:
    let
      op = { acc, state }: { name, value }:
        let
          step = f name value state;
        in {
          acc = acc ++ [
            {
              inherit name;
              inherit (step) value;
            }
          ];
          inherit (step) state;
        };
      nul = {
        acc = [];
        state = state0;
      };
      final = lib.foldl' op nul (lib.mapAttrsToList lib.nameValuePair attrs);
    in {
      attrs = lib.listToAttrs final.acc;
      inherit (final) state;
    };

  # f :: (attrPath :: [String]) -> (depName :: String) -> (depAttrs :: { ... }) -> (state :: a) -> { value :: { ... }, state :: a }
  traverseDependencies = f: manifest: state0:
    let
      dependencyAttributes = [
        "dependencies"
        "dev-dependencies"
        "build-dependencies"
      ];

      traverseTheseDependencies = attrPath: dependencies:
        let
        in
          lib.flip traverseAttrs dependencies (name: value: state':
            let
            in
              f attrPath name value state'
          );

      fForTarget = attrPath: name: value: state':
        if lib.elem name dependencyAttributes
        then
          let
            step = traverseTheseDependencies (attrPath ++ [ name ]) value state';
          in {
            value = step.attrs;
            state = step.state;
          }
        else {
          inherit value;
          state = state';
        }
      ;

      traverseTarget = attrPath: traverseAttrs (fForTarget attrPath);

      fForTargets = name: value: state':
        if name == "target"
        then
          let step =
            traverseAttrs
              (name: value: state':
                let step = traverseAttrs (fForTarget [ "target" name ]) value state';
                in {
                  value = step.attrs;
                  inherit (step) state;
                }
              )
              value
              state'
            ;
          in {
            inherit (step) state;
            value = step.attrs;
          }
        else {
          inherit value;
          state = state';
        }
      ;

      traverseTargets = traverseAttrs fForTargets;

      step = traverseTarget [] manifest state0;
    in
      traverseTargets step.attrs step.state;

  extractAndPatchPathDependencies = patch: manifest:
    let
      step = traverseDependencies
        (attrPath: depName: depAttrs: state':
          if depAttrs ? path
          then
            let
              realDepName = depAttrs.package or depName;
              consistent = if state' ? "${realDepName}" then state'."${realDepName}" == depAttrs.path else true;
            in
              assert consistent;
              {
                value = depAttrs // {
                  path = patch realDepName depAttrs.path;
                };
                state = state' // {
                  "${realDepName}" = depAttrs.path;
                };
              }
          else
            {
              value = depAttrs;
              state = state';
            }
        )
        manifest
        {}
      ;
    in {
      patchedManifest = step.attrs;
      pathDependencies = step.state;
    };

  ###

  crateManifest = cratePath: builtins.fromTOML (builtins.readFile (crateManifestPath cratePath));
  crateManifestPath = cratePath: cratePath + "/Cargo.toml";
  crateSrcPath = cratePath: cratePath + "/src";

  mkCrate =
    cratePath:

    { extraPaths ? []
    , resolveLinks ? false
    }:

    let
      manifest = crateManifest cratePath;

      inherit (manifest.package) name version;

      hasImplicitBuildScript = builtins.pathExists (cratePath + "/build.rs");
      hasExplicitBuildScript = manifest.package ? "build";
      hasAnyBuildScript = hasImplicitBuildScript || hasExplicitBuildScript;

      extractedAndPatched = extractAndPatchPathDependencies (realDepName: pathValue: "../${realDepName}") manifest;
      pathDependencies = extractedAndPatched.pathDependencies;
      manifestWithPatchedPathDependencies = extractedAndPatched.patchedManifest;

      realPatchedManifest = manifestWithPatchedPathDependencies;

      dummyPatchedManifest = clobber [
        (removeAttrs manifestWithPatchedPathDependencies [
          "test"
          "bench"
          "example"
        ])
        (lib.optionalAttrs hasAnyBuildScript {
          package.build = dummyMainWithStdInSrc;
        })
        {
          lib = {
            path = dummyLibInSrc;
            harness = false;
          };
        }
        {
          bin = lib.forEach (manifest.bin or []) (bin:
            bin // {
              path = dummyMainWithOrWithoutStdInSrc;
            }
          );
        }
      ];

      real = if resolveLinks then realResolved else realUnresolved;

      realResolved = runCommand realUnresolved.name {} ''
        cp -rL ${realUnresolved} $out
      '';

      realUnresolved = linkFarm "real-crate-${name}" ([
        (rec {
          name = "Cargo.toml";
          path = toTOMLFile name realPatchedManifest;
        })
        { name = "src";
          path = crateSrcPath cratePath;
        }
      ] ++ (lib.optionals hasImplicitBuildScript [
        { name = "build.rs";
          path = cratePath + "/build.rs";
        }
      ]) ++ (map
        (path: {
          name = path;
          path = cratePath + "/${path}";
        })
        extraPaths
      ));

      dummy = linkFarm "dummy-crate-${name}" [
        (rec {
          name = "Cargo.toml";
          path = toTOMLFile name dummyPatchedManifest;
        })
      ];

    in {
      inherit name version manifest real dummy pathDependencies;
    };

  augmentCrates = crates: lib.fix (selfCrates:
    lib.flip lib.mapAttrs crates (_: crate:
      let
        pathDependenciesList = lib.mapAttrsToList (depName: _: selfCrates.${depName}) crate.pathDependencies;
      in
        lib.fix (selfCrate: crate // {
          closure = {
            "${crate.name}" = selfCrate;
          } // lib.foldl' (acc: crate': acc // crate'.closure) {} pathDependenciesList;
        })
    )
  );

}
