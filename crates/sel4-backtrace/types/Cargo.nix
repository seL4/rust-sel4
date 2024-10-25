#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, versions, serdeWith, postcardWith, localCrates }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-backtrace-types";
  package.license = "MIT";
  dependencies = {
    inherit (versions) cfg-if;
    serde = serdeWith [ "derive" ] // { optional = true; };
    postcard = postcardWith [] // { optional = true; };
    addr2line = { version = versions.addr2line; default-features = false; features = [ "rustc-demangle" "cpp_demangle" "fallible-iterator" "smallvec" ]; optional = true; };
    sel4-backtrace-symbolize = localCrates.sel4-backtrace-symbolize // { optional = true; };
  };
  features = {
    alloc = [
      "serde?/alloc"
    ];
    serde = [
      "dep:serde"
    ];
    postcard = [
      "serde"
      "dep:postcard"
    ];
    symbolize = [
      "addr2line"
      "alloc"
      "sel4-backtrace-symbolize"
    ];
    full = [
      "alloc"
      "serde"
      "postcard"
      "symbolize"
    ];
  };
}
