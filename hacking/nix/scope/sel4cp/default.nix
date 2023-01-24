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

  # sel4cpSource = lib.cleanSource ../../../../../../../../x/sel4cp;

  sel4cpSource =
    let
      rev = "8e4c0d3719c490bf0955f9207abdab210af5b2a6";
      ref = "refs/tags/keep/${builtins.substring 0 32 rev}";
    in
      builtins.fetchGit {
        url = "https://gitlab.com/coliasgroup/sel4cp.git";
        inherit rev ref;
      };

  kernelSource = builtins.fetchGit {
    url = "https://github.com/BreakawayConsulting/seL4.git";
    rev = "92f0f3ab28f00c97851512216c855f4180534a60";
    ref = "sel4cp-core-support";
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
