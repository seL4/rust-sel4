{ runCommand
, capdl-tool
, objectSizes
, sources
}:

{ spec }:

let
  # exe = "parse-capDL";
  exe = sources.localRoot + "/capdl/capDL-tool/parse-capDL";
in

runCommand "spec.json" {
  nativeBuildInputs = [
    capdl-tool

    # HACK HACK HACK
    (import <nixpkgs> {}).stack
  ];
} ''
  ${exe} --object-sizes=${objectSizes} --json=$out ${spec}
''
