#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, runCommand, linkFarm
, buildPackages
, fetchurl
, jq
, toTOMLFile
, defaultRustToolchain
, crateUtils
}:

vendoredLockfiles:

let
  getSingleton = name: attrs:
    assert lib.attrNames attrs == [ name ];
    attrs.${name};

  sourceValues = lib.forEach vendoredLockfiles (attrs: getSingleton "source" attrs.configFragment);

  configFragment = {
    source = lib.mapAttrs mergeSource (lib.zipAttrs sourceValues);
  };

  mergeSource = name: values:
    let
      first = lib.head values;
    in
      if lib.all (x: x == first) values
      then first
      else
        let
          directories = lib.forEach values (getSingleton "directory");
        in {
          directory = mergeDirectories directories;
        };

  mergeDirectories = directories: runCommand "vendored" {} ''
    mkdir $out
    for d in ${lib.concatMapStringsSep " " (x: "${x}/*") directories}; do
      name=$(basename $d)
      if [ -d $out/$name ]; then
        ! diff -r -q $d $out/$name
      else
        cp -r $d $out
      fi
    done
  '';

in {
  inherit configFragment;
}
