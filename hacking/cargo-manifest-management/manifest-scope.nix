#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib }:

let

  filterOutEmptyFeatureList = attrs:
    builtins.removeAttrs attrs (lib.optional ((attrs.features or null) == []) "features");

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
    addr2line = "0.21.0";
    anyhow = "1.0.66";
    async-unsync = "0.2.2";
    cfg-if = "1.0.0";
    clap = "4.4.6";
    fallible-iterator = "0.2.0";
    futures = "0.3.28";
    getrandom = "0.2.10";
    gimli = "0.28.0";
    heapless = "0.7.16";
    log = "0.4.17";
    num = "0.4.1";
    num_enum = "0.5.9";
    num-traits = "0.2.16";
    object = "0.32.1";
    pin-project = "1.1.3";
    postcard = "1.0.2";
    proc-macro2 = "1.0.50";
    quote = "1.0.23";
    rand = "0.8.5";
    serde = "1.0.147";
    serde_json = "1.0.87";
    serde_yaml = "0.9.14";
    smoltcp = "0.10.0";
    syn = "1.0.107";
    synstructure = "0.12.6";
    tock-registers = "0.8.1";
    unwinding = "0.1.6";
    virtio-drivers = "0.5.0";
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
    tag = "keep/8aee5539716d3d38247f46eddd42b382"; # branch coliasgroup
  };

  fatSource = {
    git = "https://github.com/coliasgroup/rust-embedded-fat.git";
    tag = "keep/e1465a43c9f550ef58701a275b313310"; # branch sel4
  };

  mbedtlsSource = {
    git = "https://github.com/coliasgroup/rust-mbedtls";
    tag = "keep/30d001b63baea36135b2590c4fd05e95"; # branch sel4
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
}
