{ lib, stdenv, writeText, buildPackages
, cmake, ninja
, dtc, libxml2
, python3Packages
, qemu
, sources
}:

kernelConfig:

let
  src = sources.fetchGit {
    url = "https://gitlab.com/coliasgroup/seL4.git";
    rev = "862b34791e2e3720bdafc74395469c1b4b97807b"; # branch "rust"
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
