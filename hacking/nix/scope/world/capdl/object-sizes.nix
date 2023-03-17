{ runCommandCC, sources, libsel4 }:

runCommandCC "object_sizes.yaml" {
  buildInputs = [ libsel4 ];
} ''
  $CC -E -P - < ${sources.objectSizes}/object_sizes.yaml > $out
''
