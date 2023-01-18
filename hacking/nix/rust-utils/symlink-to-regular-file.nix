{ runCommand }:

name: path:

runCommand name {} ''
  cp -L ${path} $out
''
