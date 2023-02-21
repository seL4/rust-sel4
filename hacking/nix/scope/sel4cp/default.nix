{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, writeScript, runCommand
, callPackage
, cmake, ninja
, dtc, libxml2
, python3Packages
, qemu
, newlib
}:

let
  mkKeepRef = rev: "refs/tags/keep/${builtins.substring 0 32 rev}";

  # sel4cpSource = lib.cleanSource ../../../../../../../../x/sel4cp;

  sel4cpSource = builtins.fetchGit rec {
    url = "https://gitlab.com/coliasgroup/sel4cp.git";
    rev = "e55891080f408f8fa29ae3ab674a04c56adbbb01";
    ref = mkKeepRef rev;
  };

  # kernelSource = lib.cleanSource ../../../../../../../../x/seL4;

  kernelSource = builtins.fetchGit rec {
    url = "https://gitlab.com/coliasgroup/seL4.git";
    rev = "f23b954d43782f2d83a77580fded50088a8dec00";
    ref = mkKeepRef rev;
  };

  kernelSourcePatched = stdenv.mkDerivation {
    name = "kernel-source-for-sel4cp";
    src = kernelSource;
    phases = [ "unpackPhase" "patchPhase" "installPhase" ];
    nativeBuildInputs = [
      python3Packages.sel4-deps
    ];
    postPatch = ''
      # patchShebangs can't handle env -S
      rm configs/*_verified.cmake

      patchShebangs --build .
    '';
    installPhase = ''
      cp -R ./ $out
    '';
  };

  libc =
    let
      root = "${newlib}/${stdenv.hostPlatform.config}";
    in
      linkFarm "libc" [
        { name = "include"; path = "${root}/include"; }
        { name = "lib"; path = "${root}/lib"; }
      ];

  sdk = stdenv.mkDerivation {
    name = "sel4cp-sdk";

    src = lib.cleanSourceWith {
      src = sel4cpSource;
      filter = name: type:
        let baseName = baseNameOf (toString name);
        in !(type == "directory" && baseName == "tool");
    };

    buildInputs = [
      libc
    ];

    nativeBuildInputs = [
      cmake ninja
      dtc libxml2
      python3Packages.sel4-deps
    ];

    depsBuildBuild = [
      # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
      qemu
    ];

    dontConfigure = true;
    dontFixup = true;

    buildPhase = ''
      python3 build_sdk.py --sel4=${kernelSourcePatched}
    '';

    installPhase = ''
      mv release/sel4cp-sdk-1.2.6 $out
    '';
  };

  tool = linkFarm "sel4cp-tool" [
    (rec { name = "sel4coreplat"; path = lib.cleanSource (sel4cpSource + "/tool/${name}"); })
  ];

  exampleSource = sel4cpSource + "/example/qemu_arm_virt/hello";

  examplePDs = stdenv.mkDerivation {
    name = "example";

    src = exampleSource;

    dontConfigure = true;
    dontFixup = true;

    buildInputs = [
      libc
    ];

    nativeBuildInputs = [
      python3Packages.sel4-deps
    ];

    SEL4CP_SDK = sdk;
    SEL4CP_BOARD = "qemu_arm_virt";
    SEL4CP_CONFIG = "debug";

    SEL4CP_TOOL = "python3 -m sel4coreplat";

    buildPhase = ''
      export PYTHONPATH=${tool}:$PYTHONPATH
      mkdir build
      make BUILD_DIR=build
    '';

    installPhase = ''
      mkdir $out
      mv build/hello.elf $out
    '';
  };

  mkSystem = { searchPath, systemXML }:
    lib.fix (self: runCommand "system" {
      SEL4CP_SDK = sdk;
      SEL4CP_BOARD = "qemu_arm_virt";
      SEL4CP_CONFIG = "debug";

      nativeBuildInputs = [
        python3Packages.sel4-deps
      ];

      passthru = rec {
        loader = "${self}/loader.img";
        simulate = mkSimulate loader;
        links = linkFarm "links" [
          { name = "simulate"; path = simulate; }
          { name = "pds"; path = searchPath; }
          { name = "loader.elf"; path = loader; }
          { name = "report.txt"; path = "${self}/report.txt"; }
          { name = "sdk/monitor.elf"; path = "${sdk}/board/qemu_arm_virt/debug/elf/monitor.elf"; }
          { name = "sdk/loader.elf"; path = "${sdk}/board/qemu_arm_virt/debug/elf/loader.elf"; }
        ];
      };
    } ''
      export PYTHONPATH=${tool}:$PYTHONPATH
      mkdir $out
	    python3 -m sel4coreplat ${systemXML} \
        --search-path ${searchPath} \
        --board $SEL4CP_BOARD \
        --config $SEL4CP_CONFIG \
        -o $out/loader.img \
        -r $out/report.txt
    '');

  example = mkSystem {
    searchPath = examplePDs;
    systemXML = exampleSource + "/hello.system";
  };

  mkSimulate = loader: writeScript "x.sh" ''
    #!${buildPackages.runtimeShell}
    exec ${pkgsBuildBuild.qemu}/bin/qemu-system-aarch64 \
      -machine virt \
      -cpu cortex-a53 -m size=1G \
      -device loader,file=${loader},addr=0x70000000,cpu-num=0 \
      -serial mon:stdio \
      -nographic \
      "$@"
  '';

  pyoxidizer = callPackage ./pyoxidizer.nix {};
  pyoxidizerBroken = callPackage ./pyoxidizer-broken.nix {};

in rec {
  inherit
    sdk tool
    mkSystem mkSimulate
    pyoxidizer pyoxidizerBroken
    example
  ;
}
