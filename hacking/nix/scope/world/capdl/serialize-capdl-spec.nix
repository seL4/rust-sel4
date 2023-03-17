{ runCommand
, capdl-tool
, objectSizes
}:

{ spec }:

let
  exe = "parse-capDL";
in

runCommand "spec.json" {
  nativeBuildInputs = [
    capdl-tool
  ];
} ''
  ${exe} --object-sizes=${objectSizes} --json=$out ${spec}
''
