#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, versions }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-backtrace-symbolize";
  package.license = "MIT OR Apache-2.0";
  dependencies = {
    addr2line = {
      version = versions.addr2line;
      default-features = false;
      features = [ "rustc-demangle" "cpp_demangle" "fallible-iterator" "smallvec" ];
    };
  };
}
