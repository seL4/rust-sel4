#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mkDerivation, base, base-compat, hspec, hspec-discover, lib
, QuickCheck
}:
mkDerivation {
  pname = "base-compat-batteries";
  version = "0.11.2";
  sha256 = "31e066a5aa96af94fe6465adb959c38d63a49e01357641aa4322c754a94d3023";
  libraryHaskellDepends = [ base base-compat ];
  testHaskellDepends = [ base hspec QuickCheck ];
  testToolDepends = [ hspec-discover ];
  description = "base-compat with extra batteries";
  license = lib.licenses.mit;
}
