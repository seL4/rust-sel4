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

  versions = {
    addr2line = "0.24.2";
    anyhow = "1.0.66";
    async-unsync = "0.2.2";
    cfg-if = "1.0.0";
    chrono = "0.4.35";
    clap = "4.4.6";
    dlmalloc = "0.2.3";
    embedded-hal-nb = "1.0";
    embedded-io-async = "0.6.1";
    fallible-iterator = "0.2.0";
    fdt = "0.1.5";
    futures = "0.3.28";
    getrandom = "0.2.10";
    gimli = "0.31.1";
    heapless = "0.7.16";
    lock_api = "0.4.12";
    log = "0.4.17";
    num = "0.4.1";
    num_enum = "0.5.9";
    num-traits = "0.2.16";
    object = "0.36.5";
    pin-project = "1.1.3";
    postcard = "1.0.2";
    proc-macro2 = "1.0.89";
    quote = "1.0.37";
    rand = "0.8.5";
    rtcc = "0.3.2";
    rustc_version = "0.4.0";
    rustls = "0.23.5";
    serde = "1.0.147";
    serde_json = "1.0.87";
    serde_yaml = "0.9.14";
    smoltcp = "0.10.0";
    syn = "2.0.85";
    thiserror = "1.0";
    tock-registers = "0.8.1";
    unwinding = "0.2.1";
    virtio-drivers = "0.7.2";
    webpki-roots = "0.26";
    zerocopy = "0.7.32";
  };

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
    version = "=0.17.8";
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
    tag = mkKeepRef "c1d8b986315b1d7fcaa0bf63c2e0497fbebab231"; # branch dev
  };
}
