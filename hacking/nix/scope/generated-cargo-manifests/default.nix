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
    # "mbedtls" = "tmp/rust-mbedtls/mbedtls";
    # "mbedtls-sys-auto" = "tmp/rust-mbedtls/mbedtls-sys";
    # "mbedtls-platform-support" = "tmp/rust-mbedtls/mbedtls-platform-support";
    # "embedded-fat" = "tmp/rust-embedded-fat";
  };

  cratePathAttrs = localCratePathAttrs // externalCratePathAttrs;

  cratePaths = lib.mapAttrs (name: path: { inherit name path; }) cratePathAttrs;

  filterOutEmptyFeatureList = attrs:
    builtins.removeAttrs attrs (lib.optional (attrs ? features && attrs.features == []) "features");

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
        addr2line = "0.21.0";
        anyhow = "1.0.66";
        cfg-if = "1.0.0";
        fallible-iterator = "0.2.0";
        futures = "0.3.28";
        gimli = "0.28.0";
        heapless = "0.7.16";
        log = "0.4.17";
        num = "0.4.1";
        num-derive = "0.4.0";
        num-traits = "0.2.16";
        num_enum = "0.5.9";
        object = "0.32.1";
        postcard = "1.0.2";
        proc-macro2 = "1.0.50";
        quote = "1.0.23";
        serde = "1.0.147";
        serde_json = "1.0.87";
        serde_yaml = "0.9.14";
        smoltcp = "0.10.0";
        syn = "1.0.107";
        synstructure = "0.12.6";
        tock-registers = "0.8.1";
        unwinding = "0.1.6";
        virtio-drivers = "0.5.0";
        zerocopy = "0.6.4";
      };

      mbedtlsSource = {
        git = "https://github.com/coliasgroup/rust-mbedtls";
        tag = "keep/6eef662dbf636d1ccf86078143b6854f"; # branch sel4
      };

      mbedtlsWith = features: filterOutEmptyFeatureList (mbedtlsSource // {
        default-features = false;
        features = [ "no_std_deps" ] ++ features;
      });

      mbedtlsSysAutoWith = features: filterOutEmptyFeatureList (mbedtlsSource // {
        default-features = false;
        inherit features;
      });

      mbedtlsPlatformSupportWith = features: filterOutEmptyFeatureList (mbedtlsSource // {
        default-features = false;
        inherit features;
      });

      serdeWith = features: filterOutEmptyFeatureList {
        version = versions.serde;
        default-features = false;
        inherit features;
      };

      postcardWith = features: filterOutEmptyFeatureList {
        version = versions.postcard;
        default-features = false;
        inherit features;
      };

      unwindingWith = features: filterOutEmptyFeatureList {
        version = versions.unwinding;
        default-features = false;
        features = [ "unwinder" "fde-custom" "hide-trace" ] ++ features;
      };

      smoltcpWith = features: filterOutEmptyFeatureList {
        version = versions.smoltcp;
        default-features = false;
        features = smoltcpBaseProtosFeatures ++ features;
      };

      smoltcpWithAdjustedDefaultsAnd = features:
        smoltcpWith (smoltcpAdjustedDefaultFeatures ++ features);

      smoltcpBaseProtosFeatures = [
        "proto-ipv4"
        "proto-dhcpv4"
        "proto-dns"
        "socket-dhcpv4"
        "socket-dns"
        "socket-tcp"
      ];

      # default features with:
      #   - "std"
      #   + "alloc"
      #   + "phy-raw_socket"
      #   + "phy-tuntap_interface"
      smoltcpAdjustedDefaultFeatures = [
        "alloc" "log"
        "medium-ethernet" "medium-ip" "medium-ieee802154"
        
        "proto-ipv4" "proto-igmp" "proto-dhcpv4" "proto-ipv6" "proto-dns"
        "proto-ipv4-fragmentation" "proto-sixlowpan-fragmentation"
        "socket-raw" "socket-icmp" "socket-udp" "socket-tcp" "socket-dhcpv4" "socket-dns" "socket-mdns"
        "packetmeta-id" "async"
      ];

      fatSource = {
        git = "https://github.com/coliasgroup/rust-embedded-fat.git";
        tag = "keep/e1465a43c9f550ef58701a275b313310"; # branch sel4
      };

      virtioDriversWith = features: filterOutEmptyFeatureList {
        version = versions.virtio-drivers;
        # git = "https://github.com/rcore-os/virtio-drivers.git";
        # rev = "7385d61153aff3236d7083d143bbeef9c2eb7326"; # first bad
        # rev = "cfb8a80cb64c2382586dd9919a82ad40bdbd4892"; # bad
        # rev = "a2d79f1a0a0e98a33257ced9d151828feecfd23c"; # good
        # rev = "bc78b54f32b39bc3aeda4f43dbd17f51d665f3d9"; # good
        default-features = false;
        inherit features;
      };
    };

  workspaceTOML = helpers.renderManifest {
    frontmatter = null;
    manifest = {
      workspace = {
        resolver = "2";
        default-members = [];
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
    # "support/workspace-meta.json" = {
    #   src = metaJSON;
    #   justCheckEquivalenceWith = null;
    # };
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
