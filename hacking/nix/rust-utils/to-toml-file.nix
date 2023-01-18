{ runCommand, remarshal }:

name: expr:

runCommand name {
  nativeBuildInputs = [
    remarshal
  ];
  json = builtins.toJSON expr;
  passAsFile = [ "json" ];
} ''
  remarshal -if json -of toml -i $jsonPath -o $out
''
