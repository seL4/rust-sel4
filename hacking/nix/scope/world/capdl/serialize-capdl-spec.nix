#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand
, capdl-tool
, objectSizes
, sources
}:

{ cdl }:

let
  exe = "parse-capDL";
  # exe = sources.localRoot + "/capdl/capDL-tool/parse-capDL";
in

runCommand "spec.json" {
  nativeBuildInputs = [
    capdl-tool

    # HACK HACK HACK
    # (import <nixpkgs> {}).stack
  ];
} ''
  ${exe} --object-sizes=${objectSizes} --json=$out ${cdl}
''
