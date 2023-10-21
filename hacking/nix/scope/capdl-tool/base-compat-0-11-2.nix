#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mkDerivation, base, lib, unix }:
mkDerivation {
  pname = "base-compat";
  version = "0.11.2";
  sha256 = "53a6b5145442fba5a4bad6db2bcdede17f164642b48bc39b95015422a39adbdb";
  revision = "1";
  editedCabalFile = "0h6vr19vr5bhm69w8rvswbvd4xgazggkcq8vz934x69www2cpgri";
  libraryHaskellDepends = [ base unix ];
  description = "A compatibility layer for base";
  license = lib.licenses.mit;
}
