#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib }:

let

  filterOutEmptyFeatureList = attrs:
    builtins.removeAttrs attrs (lib.optional ((attrs.features or null) == []) "features");

  mkKeepRef = rev: "keep/${builtins.substring 0 32 rev}";

in rec {
  inherit lib;

  mk = args: (lib.recursiveUpdate
    {
      nix.frontmatter = defaultFrontmatter;
      package = {
        edition = "2021";
        version = "0.1.0";
        license = defaultLicense;
        authors = defaultAuthors;
      };
    }
    args
  );

  defaultFrontmatter = mkDefaultFrontmatterWithReuseArgs defaultReuseFrontmatterArgs;

  mkDefaultFrontmatterWithReuseArgs = args: mkReuseFrontmatter args + defaultNoteFrontmatter;

  overrideDefaultFrontmatter =
    { copyrightLines ? defaultReuseFrontmatterArgs.copyrightLines ++ extraCopyrightLines
    , extraCopyrightLines ? []
    , licenseID ? defaultReuseFrontmatterArgs.licenseID
    }:
    mkDefaultFrontmatterWithReuseArgs {
      inherit copyrightLines licenseID;
    };

  defaultNoteFrontmatter = ''
    #
    # This file is generated from './Cargo.nix'. You can edit this file directly
    # if you are not using this project's Cargo manifest management tools.
    # See 'hacking/cargo-manifest-management/README.md' for more information.
    #
  '';

  mkReuseFrontmatter = { copyrightLines, licenseID }: ''
    #
  '' + lib.flip lib.concatMapStrings copyrightLines (line: ''
    # ${line}
  '') + ''
    #
    # ${spdxLicenseIdentifierMagic} ${licenseID}
    #
  '';

  # REUSE-IgnoreStart
  spdxLicenseIdentifierMagic = "SPDX-License-Identifier:";
  # REUSE-IgnoreEnd

  defaultReuseFrontmatterArgs = {
    copyrightLines = defaultCopyrightLines;
    licenseID = defaultLicense;
  };

  defaultCopyrightLines = with copyrightLines; [
    coliasgroup
  ];

  defaultLicense = "BSD-2-Clause";

  defaultAuthors = with authors; [
    nspin
  ];

  authors = {
    nspin = "Nick Spinale <nick.spinale@coliasgroup.com>";
  };

  copyrightLines = {
    coliasgroup = "Copyright 2023, Colias Group, LLC";
  };

  versions =
    let
      table = builtins.fromTOML (builtins.readFile ./direct-dependency-allow-list.toml);
      getVersion = v: if lib.isString v then v else v.version;
    in
      lib.mapAttrs (lib.const getVersion) table.allow;

  zerocopyWith = features: filterOutEmptyFeatureList {
    version = versions.zerocopy;
    inherit features;
  };

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

  volatileSource = {
    git = "https://github.com/coliasgroup/volatile.git";
    tag = mkKeepRef "aa7512906e9b76066ed928eb6986b0f9b1750e91"; # branch coliasgroup
  };

  fatSource = {
    git = "https://github.com/coliasgroup/rust-embedded-fat.git";
    tag = mkKeepRef "e1465a43c9f550ef58701a275b3133105deb9183"; # branch sel4
  };

  ringWith = features: {
    version = versions.ring;
    features = [
      "less-safe-getrandom-custom-or-rdrand"
    ] ++ features;
  };

  rustlsWith = features: {
    version = versions.rustls;
    default-features = false;
    features = [
      "logging"
      "ring"
      "tls12"
    ] ++ features;
  };

  virtioDriversWith = features: filterOutEmptyFeatureList {
    version = versions.virtio-drivers;
    default-features = false;
    inherit features;
  };

  dafnySource = {
    git = "https://github.com/coliasgroup/dafny.git";
    tag = mkKeepRef "02d0a578fdf594a38c7c72d7ad56e1a62e464f44"; # branch dev
  };

  verusSource = {
    git = "https://github.com/coliasgroup/verus.git";
    tag = mkKeepRef "a60b3b7e08d514dc8e0ac3b30236db0c3023c17a"; # branch dev
  };
}
