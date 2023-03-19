{ lib, stdenv, writeText, buildPackages
, cmake, ninja
, dtc, libxml2
, python3Packages
, qemu
, kernelConfig
, mkKeepRef
}:

let
  # src = lib.cleanSource ../../../../../../../../x/seL4;

  src = builtins.fetchGit rec {
    url = "https://gitlab.com/coliasgroup/seL4.git";
    rev = "c7f53af3de8dc1e2fd7243aea0e6f85a75bd8fbf"; # branch "rust"
    ref = mkKeepRef rev;
  };

  settings = writeText "settings.cmake" ''
    ${lib.concatStrings (lib.mapAttrsToList (k: v: ''
      set(${k} ${v.value} CACHE ${v.type} "")
    '') kernelConfig)}
  '';

in
stdenv.mkDerivation {
  name = "seL4";

  inherit src;

  nativeBuildInputs = [
    cmake ninja
    dtc libxml2
    python3Packages.sel4-deps
  ];

  depsBuildBuild = [
    # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
    qemu
  ];

  hardeningDisable = [ "all" ];

  postPatch = ''
    # patchShebangs can't handle env -S
    rm configs/*_verified.cmake

    patchShebangs --build .
  '';

  configurePhase = ''
    build=$(pwd)/build

    cmake \
      -DCROSS_COMPILER_PREFIX=${stdenv.cc.targetPrefix} \
      -DCMAKE_TOOLCHAIN_FILE=gcc.cmake \
      -DCMAKE_INSTALL_PREFIX=$out \
      -C ${settings} \
      -G Ninja \
      -B $build
  '';

  buildPhase = ''
    ninja -C $build all
  '';

  installPhase = ''
    ninja -C $build install
  '';

  dontFixup = true;
}
