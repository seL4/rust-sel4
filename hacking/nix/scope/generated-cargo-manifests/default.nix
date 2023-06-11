{ lib, callPackage, newScope
, writeText, runCommand, linkFarm, writeScript
, runtimeShell
, python3, jq
, crateUtils
, cargoManifestGenrationUtils
, sources
}:

let
  helpers = cargoManifestGenrationUtils;

  inherit (helpers) mkCrate;

  cargoManifestSourcesRoot = sources.srcRoot + "/hacking/cargo-manifest-sources";

  cargoManifestSources =
    let
      f = parent: lib.concatLists (lib.mapAttrsToList (name: type:
        let
          child = "${parent}/${name}";
        in {
          "regular" = lib.optional (name == "crate.nix") child;
          "directory" = f child;
        }."${type}" or []
      ) (builtins.readDir parent));
    in
      f (toString cargoManifestSourcesRoot);

  crates = lib.listToAttrs (lib.forEach cargoManifestSources (path:
    let
      relativePath =
        helpers.pathBetween
          (toString cargoManifestSourcesRoot)
          (toString (builtins.dirOf path));

      crate = callCrate { inherit relativePath; } path {};
    in {
      name = crate.manifest.package.name;
      value = crate;
    }));

  localCratePathAttrs = lib.mapAttrs (_: crate: crate.path) crates;

  externalCratePathAttrs = {
    # "virtio-drivers" = "tmp/virtio-drivers";
  };

  cratePathAttrs = localCratePathAttrs // externalCratePathAttrs;

  cratePaths = lib.mapAttrs (name: path: { inherit name path; }) cratePathAttrs;

  callCrate = { relativePath }:

    newScope rec {

      localCrates = cratePaths;

      mk = args: mkCrate (crateUtils.clobber [
        {
          nix.path = relativePath;
          package = {
            edition = "2021";
            version = "0.1.0";
            license = defaultLicense;
            authors = [ defaultAuthor ];
          };
        }
        args
      ]);

      defaultLicense = "BSD-2-Clause";
      defaultAuthor = "Nick Spinale <nick.spinale@coliasgroup.com>";

      versions = {
        anyhow = "1.0.66";
        cfg-if = "1.0.0";
        fallible-iterator = "0.2.0";
        futures = "0.3.28";
        heapless = "0.7.16";
        log = "0.4.17";
        num_enum = "0.5.9";
        object = "0.31.0";
        postcard = "1.0.2";
        proc-macro2 = "1.0.50";
        quote = "1.0.23";
        serde = "1.0.147";
        serde_json = "1.0.87";
        serde_yaml = "0.9.14";
        syn = "1.0.107";
        synstructure = "0.12.6";
        tock-registers = "0.8.1";
        unwinding = "0.1.6";
        zerocopy = "0.6.1";
      };

      serdeWith = features: {
        version = versions.serde;
        default-features = false;
        inherit features;
      };

      postcardWith = features: {
        version = versions.postcard;
        default-features = false;
        inherit features;
      };

      unwindingWith = features: {
        version = versions.unwinding;
        default-features = false;
        features = [ "unwinder" "fde-custom" "hide-trace" ] ++ features;
      };
    };

  workspaceTOML = helpers.renderManifest {
    frontmatter = null;
    manifest = {
      workspace = {
        resolver = "2";
        members = lib.naturalSort (lib.attrValues cratePathAttrs);
      };
    };
  };

  meta = lib.mapAttrs (_: crate: crate.meta) crates;

  metaJSON = runCommand "workspace-meta.json" {
    nativeBuildInputs = [
      jq
    ];
    json = builtins.toJSON meta;
    passAsFile = [ "json" ];
  } ''
    jq . $jsonPath > $out
  '';

  # for manual inspection, useful for hacking on this script
  links = linkFarm "crates" (
    lib.mapAttrsToList (k: v: {
      name = k;
      path = v.src;
    }) plan
  );

  plan = {
    "Cargo.toml" = {
      src = workspaceTOML;
      justCheckEquivalenceWith = helpers.checkTOMLEquivalence;
    };
    "support/workspace-meta.json" = {
      src = metaJSON;
      justCheckEquivalenceWith = null;
    };
  } // lib.mapAttrs' (_: crate: {
    name = "${crate.path}/Cargo.toml";
    value = {
      src = crate.manifestTOML;
      justCheckEquivalenceWith = if crate.justEnsureEquivalence then helpers.checkTOMLEquivalence else null;
    };
  }) crates;

  script = actuallyDoIt: helpers.mkScript {
    inherit actuallyDoIt plan;
    root = toString sources.srcRoot;
  };

  update = script true;
  check = script false;

in {
  inherit crates workspaceTOML meta metaJSON links;
  inherit update check;
}
