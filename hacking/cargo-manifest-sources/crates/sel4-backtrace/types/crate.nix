#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT
#

{ mk, versions, serdeWith, postcardWith }:

mk rec {
  package.name = "sel4-backtrace-types";
  package.license = "MIT";
  nix.reuseFrontmatterArgs.licenseID = package.license;
  dependencies = {
    inherit (versions) cfg-if;
    serde = serdeWith [ "derive" ] // { optional = true; };
    postcard = postcardWith [] // { optional = true; };
    addr2line = { version = versions.addr2line; default-features = false; features = [ "rustc-demangle" "cpp_demangle" "object" "fallible-iterator" "smallvec" ]; optional = true; };
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
    ];
    full = [
      "alloc"
      "serde"
      "postcard"
      "symbolize"
    ];
  };
}
