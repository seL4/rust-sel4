#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-backtrace-addr2line-context-helper";
  dependencies = {
    addr2line = { version = versions.addr2line; default-features = false; features = [ "rustc-demangle" "cpp_demangle" "fallible-iterator" "smallvec" ]; };
    gimli = { version = versions.gimli; default-features = false; features = [ "endian-reader" ]; };
    object = { version = versions.object; default-features = false; features = [ "read" ]; };
    stable_deref_trait = { version = "1.1.0"; default-features = false; features = [ "alloc" ]; };
  };
}
