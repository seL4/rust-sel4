{ lib, stdenv, writeText, buildPackages
, cmake, ninja
, dtc, libxml2
, python3Packages
, qemu
, kernelConfig
}:

let
  src =
    let
      rev = "0c9a1980867f715c1d06e53b5fbb6bac4a88845e";
      ref = "refs/tags/keep/${builtins.substring 0 32 rev}";
    in
      builtins.fetchGit {
        url = "https://gitlab.com/coliasgroup/seL4.git";
        inherit rev ref;
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
    cp_if_exists() {
      src=$1
      dst=$2
      if [ -e $src ]; then 
        cp $src $dst
      fi
    }

    ninja -C $build install

    mkdir $out/support

    cp $build/gen_config/kernel/gen_config.json $out/support/config.json
    cp_if_exists $build/kernel.dtb $out/support/kernel.dtb
    cp_if_exists $build/gen_headers/plat/machine/platform_gen.yaml $out/support/platform-info.yaml
  '';

  dontFixup = true;
}
