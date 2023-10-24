#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "banscii-assistant-core";
  dependencies = {
    inherit (versions) log;
    ab_glyph = { version = "0.2.22"; default-features = false; features = [ "libm" ]; };
    num-traits = { version = versions.num-traits; default-features = false; features = [ "libm" ]; };
  };
}
