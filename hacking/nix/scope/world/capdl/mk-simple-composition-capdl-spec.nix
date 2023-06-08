{ lib, buildPackages, runCommand, writeText, linkFarm
, hostPlatform
, python3Packages

, sources

, seL4ForUserspace
, objectSizes

, sel4-simple-task-serialize-runtime-config
}:

{ config
, script ? null
, command ? "python3 ${script}"
, computeUtCovers ? false
, extraNativeBuildInputs ? []
}:

let
  augmentedConfig = config // {
    kernel_config = "${seL4ForUserspace}/libsel4/include/kernel/gen_config.json";
    device_tree = if hostPlatform.isx86 then null else "${seL4ForUserspace}/support/kernel.dtb";
    platform_info =  if hostPlatform.isx86 then null else "${seL4ForUserspace}/support/platform_gen.yaml";
    object_sizes = objectSizes;
    compute_ut_covers = computeUtCovers;
  };

  augmentedConfigJSON = writeText "config.json" (builtins.toJSON augmentedConfig);

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
    sel4-simple-task-serialize-runtime-config
  ] ++ (with python3Packages; [
    future six
    aenum sortedcontainers
    pyyaml pyelftools pyfdt
  ]) ++ extraNativeBuildInputs;

  PYTHONPATH_ = lib.concatStringsSep ":" [ capdlSimpleCompositionSrc capdlSrc ];

  CONFIG = augmentedConfigJSON;

  passthru = {
    config = augmentedConfig;
    cdl = {
      spec = "${self}/spec.cdl";
      fill = "${self}/links";
    };
  };

  shellHook = ''
    export PYTHONPATH=${pythonPathForShell}:$PYTHONPATH
    export OUT_DIR=$(pwd)/x

    xxx() {
      ${command}
    }
  '';

} ''
  export PYTHONPATH=$PYTHONPATH_:$PYTHONPATH
  export OUT_DIR=$out
  ${command}
'')
