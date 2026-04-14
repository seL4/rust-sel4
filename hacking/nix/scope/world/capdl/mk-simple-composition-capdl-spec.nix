#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPackages, runCommand, writeText, linkFarm
, python312Packages

, sources

, seL4ForUserspace
, objectSizes

, sel4-simple-task-runtime-config-cli
}:

{ searchDirs
, script ? null
, command ? "python3 ${script}"
, computeUtCovers ? false
, extraNativeBuildInputs ? []
, extraPythonPath ? []
}:

let
  capdlSrc = sources.pythonCapDLTool;

  capdlSimpleCompositionSrc =
    let
      name = "capdl_simple_composition";
      path = sources.srcRoot + "/hacking/src/python/${name}";
    in
      linkFarm name [
        { inherit name path; }
      ];

  # TODO
  pythonPathForShell = "";

in
lib.fix (self: runCommand "manifest" {

  nativeBuildInputs = [
    sel4-simple-task-runtime-config-cli
  ] ++ (with python312Packages; [
    future six
    aenum sortedcontainers
    pyyaml pyelftools pyfdt
  ]) ++ extraNativeBuildInputs;

  PYTHONPATH_ = lib.concatStringsSep ":" ([ capdlSimpleCompositionSrc capdlSrc ] ++ extraPythonPath);

  passthru = {
    specAttrs = {
      cdl = "${self}/spec.cdl";
      fill = "${self}/links";
    };
  };
} ''
  PYTHONPATH=$PYTHONPATH_:$PYTHONPATH
    ${command} \
      -o $out \
      --object-sizes ${objectSizes} \
      --kernel ${seL4ForUserspace} \
      ${lib.concatStrings (lib.forEach searchDirs (d: ''
        --search-dir ${d}/bin \
      ''))}
'')
